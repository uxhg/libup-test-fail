#!/usr/bin/env python3
# This script will
#   1. find client information using functions in findcli.py
#   2. clone the repo
#   3. checkout to the old version

import argparse
import logging
from pathlib import Path

import git

from common import init_logging, LOC_REPO
from findcli import get_client_info

logger = logging.getLogger(__name__)


def add_suffix(x): return x + "-clean"


def main():
    args = handle_args()
    cli = args.client
    cli_d = get_client_info(cli)
    if not cli_d:
        exit(2)

    repo = clone_co(cli_d)
    if args.cslicer:
        create_cslicer_config(LOC_REPO / cli, cli_d["sha"], Path(f"../cslicer-configs/{cli}.properties"))


def clone_co(cli_d: dict) -> git.Repo:
    url = cli_d["url"]
    sha = cli_d["sha"]
    # new_dir: Path = LOC_REPO / f"{cli}-{get_cur_time_str()}"
    new_dir: Path = LOC_REPO / add_suffix(cli_d["name"])
    if not new_dir.exists():
        Path.mkdir(new_dir)
        repo: git.Repo = clone(url, new_dir)
        checkout(repo, sha)
        return repo


def clone(url: str, dest: Path) -> git.Repo:
    logger.info(f"Clone {url} into {dest}.")
    repo: git.Repo = git.Repo.clone_from(url, dest)
    return repo


def checkout(repo: git.Repo, sha: str):
    logger.info(f"Checkout {repo.working_dir} -> {sha}.")
    repo.git.checkout(sha)


def create_cslicer_config(repo_path: Path, sha: str, file_path: Path):
    logger.info(f"Create CSlicer config for {repo_path}.")
    out_contents = {
        "repoPath": repo_path / ".git",
        "classRoot": repo_path,
        "endCommit": sha
    }
    with file_path.open('w') as outf:
        for k, v in out_contents.items():
            outf.write(f"{k} = {v}\n")


def handle_args():
    parser = argparse.ArgumentParser(description='Clone specific client and checkout to that version')
    parser.add_argument("client", metavar="CLIENT", type=str, help="client name")
    parser.add_argument('--cslicer', action="store_true", help='Generate CSlicer configuration file')
    parser.add_argument('-l', metavar='loglevel', type=str, required=False, help='logging level, default WARNING')
    args = parser.parse_args()
    init_logging(args.l)
    logger.debug(args)
    return args


if __name__ == '__main__':
    main()
