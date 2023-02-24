#!/usr/bin/env python3
import argparse
import configparser
import json
import logging
from typing import List, NamedTuple

import requests

from util import init_logging
from search_history import query_strs

logger = logging.getLogger(__name__)


def main():
    args = handle_args()
    output_file = args.o if args.o else "results.json"
    cfg_file = args.c if args.c else "config.ini"
    cfg = parse_config_ini(cfg_file)
    gh_token = get_github_token(cfg)

    r: dict = search_by_keyword(args.keyword, gh_token)
    # list_all_for_human(r, output_file)
    select_for_human(r, output_file, gh_token)


def select_for_human(r: dict, output_file: str, gh_token: str = ""):
    logger.info(f"Filter out PRs with 2 or more commits")
    items = r.get("items")
    kept = []
    for x in items:
        # rate limit for non-search API with access tokens is 5000 requests/hour
        commit_list = check_commits_of_pr(x["url"], gh_token)
        if len(commit_list) == 0:
            logger.warning(f'{x["url"]} has 0 commit.')
        elif len(commit_list) > 1:
            kept.append(x["html_url"])
    with open(output_file, 'w') as outf:
        json.dump(kept, outf, indent=2)


def list_all_for_human(r: dict, output_file: str):
    l: list = create_list_by_key(r, "html_url")
    with open(output_file, 'w') as outf:
        json.dump(l, outf, indent=2)


def create_list_by_key(results_dict: dict, key: str) -> list:
    items = results_dict.get("items")
    pr_pages = [html_url for x in items if (html_url := x.get(key))]
    return pr_pages


def search_by_keyword(search_by: str, gh_token: str = "") -> dict:
    combined = {
        "total_count": 0,
        "incomplete_results": False,
        "items": []
    }

    for pp_index in range(1, 11):
        api_issues_pr_base_url = "https://api.github.com/search/issues?"
        # search_url = f"{api_issues_pr_base_url}q={search_by}+in:comments+is:pr+author:app/dependabot+language:rust" \
        #              f"&sort=comments&order=desc&per_page=100&page={pp_index}"
        search_url = f"{api_issues_pr_base_url}q={search_by}{query_strs[-1]}&page={pp_index}"
        logger.info(search_url)

        header = {}
        add_gh_auth_into_header(header, gh_token)
        # logger.debug(header)
        r = requests.get(search_url, headers=header)
        r_contents = r.json()
        combined["total_count"] = r_contents.get("total_count")
        for k, v in r_contents.items():
            if k != "total_count":
                try:
                    combined[k] += v
                except KeyError:
                    logger.error(f'Unexpected key "{k}" in response.')
                    logger.debug(f'Key value pair: {k}: {v}')
                    continue
    return combined


class GhAPICommit(NamedTuple):
    sha: str
    author_name: str
    commit_url: str
    changed_files: List[str]


def check_commits_of_pr(issue_api_url: str, gh_token: str = "") -> List[GhAPICommit]:
    """ Request commits endpoint of github API once for each pull request to
    get SHA1, author_name and commit endpoint url for each commit.

    To avoid too many requests to Github API, we can craft the URL manually.
    E.g., for "https://api.github.com/repos/exercism/java/issues/1916"
    we should get "https://api.github.com/repos/exercism/java/pulls/1916" first,
    then "https://api.github.com/repos/exercism/java/pulls/1916/commits".

    :param issue_api_url: with Github search API, we search issue/pr using the same endpoint and using "is:pr" to filter,
    but in the response, urls still start with ".../issues/".
    :param gh_token: github access token
    :return:
    """
    comps = issue_api_url.rsplit("issues", maxsplit=1)
    try:
        commits_url = f"{comps[0]}pulls{comps[1]}/commits"
    except IndexError:
        logger.error("URL in issue search response does not include the word 'issues'.")
        return []

    header = {}
    add_gh_auth_into_header(header, gh_token)
    r = requests.get(commits_url, headers=header).json()
    try:
        commits = [GhAPICommit(x["sha"], x["commit"]["author"]["name"], x["url"], []) for x in r]
    except TypeError:
        logger.error("Response of request to commit endpoint does not include essential fields,"
                     "possibly due to API rate limit.")
        return []
    else:
        return commits


def handle_args():
    parser = argparse.ArgumentParser(description='Search GitHub PR/Issues for catching dependabot breaking things')
    parser.add_argument("keyword", metavar="KEYWORD", type=str, help="search keyword")
    parser.add_argument("-o", metavar="OUTPUT", type=str, help="Output file")
    parser.add_argument("-c", metavar="CONFIG", type=str, help="Configuration file")
    parser.add_argument('-l', metavar='LOG_LEVEL', type=str, required=False, help='logging level, default WARNING')
    args = parser.parse_args()
    init_logging(args.l)
    logger.debug(args)
    return args


def check_search_response(search_url: str, r_contents: dict):
    """ deprecated """
    count = r_contents.get("total_count")
    if not count:
        logger.error(f"No count reported in response to {search_url}")
    items = r_contents.get("items")
    if not items:
        logger.error(f"No items found in response to {search_url}")
    is_incomplete: bool = r_contents.get("incomplete_results")
    if is_incomplete is None:
        logger.warning(f"incomplete_results is missing response to {search_url}")
    elif is_incomplete is True:
        logger.warning(f"incomplete_results set to True in response to {search_url}")


def parse_config_ini(cfg_file) -> dict:
    logger.info(f'Read config from {cfg_file}')
    cfg = configparser.ConfigParser()
    cfg.read(cfg_file)
    return cfg


def get_github_token(cfg: dict) -> str:
    try:
        token = cfg['Credentials']["GithubAccessToken"]
    except TypeError:
        logger.error("Cannot get github access token from config file, refer to config.ini")
        return ""
    else:
        if len(token) == 0:
            logger.warning("Empty token")
        return token


def add_gh_auth_into_header(header: dict, gh_token: str):
    if gh_token:
        header["Authorization"] = f"token {gh_token}"
    else:
        logger.warning("No github token provided, proceeding without token, may trigger rate limits")


if __name__ == '__main__':
    # print(check_commits_of_pr("https://api.github.com/repos/exercism/java/issues/1916"))
    main()

