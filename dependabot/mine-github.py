#!/usr/bin/env python3

import os
import sys
import re
import json
import time
import argparse
import collections
import requests
from github import Github
from github.PullRequest import PullRequest
from github.Repository import Repository

SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__)) # Dir of this script
ALL_REPOS_JSON_FILE = SCRIPT_DIR + '/all-repos.json'
PR_JSON_FILE = SCRIPT_DIR + '/pr.json'

MINING_TOKENS = ["50a22379ec5e6a7a6d4474401bcde106b100307a",
                 "0ad59546325e7f92232affad477c5ba8f04add14",
                 "d5ddfdfe8330d31f2bc892b0c03315f8880dbe46",
                 "4ab94dba9ddcda34e84f11d1bf23157963a84880",
                 "f7c5753ba099608b99514b3b13f237b663201dd3",
                 "a532c950287400bcd2e4e5148743671670b42c4b",
                 "7c00b94d47182f193b194a32c60078a1a113009a",
                 "2d713bddf89797da9f690642d0212f3a177230eb",
                 "dd497057388cc30d202361225db2c063b93a6bff",
                 "5eb1f6cdd7be63def0584408857beded81bc3111"]

def parseArgs(argv):
    parser = argparse.ArgumentParser()
    parser.add_argument('--collect-all-repos', help='Collect all repos', action='store_true', required=False)
    parser.add_argument('--analyze-prs', help='Analyze pull requests', action='store_true', required=False)

    if (len(argv) == 0):
        parser.print_help()
        exit(1)
    opts = parser.parse_args(argv)
    return opts

def isDuplicate(repo, all_repos):
    name = repo['full_name']
    for r in all_repos:
        if r['full_name'] == name:
            return True
    return False

def getStarLowerBound(repos):
    star_nums = []
    for r in repos:
        star_nums.append(r['stargazers_count'])
    return min(star_nums)

def collectAllRepos(all_repos_json_file=ALL_REPOS_JSON_FILE):
    g = Github("1c5c30b8033a7a96976633dbbbd0a28282e6aea5")
    all_repos = []
    for i in range(3):
        num_of_stars_lower_bound = -1
        while num_of_stars_lower_bound == -1 or num_of_stars_lower_bound >= 100:
            for page_index in range(10):
                search_repo_url = "https://api.github.com/search/repositories?"
                if num_of_stars_lower_bound == -1:
                    search_repo_url += "q=language:java"
                else:
                    search_repo_url += "q=language:java+stars:<" + str(num_of_stars_lower_bound + 1)
                search_repo_url += "&sort=stars" + \
                                   "&order=desc" + \
                                   "&per_page=100" + \
                                   "&page=" + str(page_index + 1)
                print (search_repo_url)
                response = requests.get(search_repo_url).content.decode("utf-8")
                #print (response)
                raw_repos = json.loads(response)['items']
                for raw_repo in raw_repos:
                    if not isDuplicate(raw_repo, all_repos):
                        all_repos.append(raw_repo)
                if page_index == 9:
                    num_of_stars_lower_bound =  getStarLowerBound(raw_repos)
                    print ('Star lower bound: ' + str(num_of_stars_lower_bound))
                time.sleep(5)
    print ('Collected ' + str(len(all_repos)) + ' repos')
    # sort repos by stars, desc
    all_repos = sorted(all_repos, key=lambda x : x['stargazers_count'])
    print (len(all_repos))
    fw = open(all_repos_json_file, 'w')
    fw.write(json.dumps(all_repos))
    fw.close()

def extractAllFilesInARepoRoot(repo: str):
    return [f.path for f in repo.get_contents("")]

def extractAllFilesInARepo(repo: str):
    all_files = []
    contents = repo.get_contents("")
    while contents:
        file_content = contents.pop(0)
        if file_content.type == "dir":
            contents.extend(repo.get_contents(file_content.path))
        else:
            all_files.append(file_content)
    return all_files

def isMavenProject(repo: str):
    all_files = extractAllFilesInARepoRoot(repo)
    for f in all_files:
        if f.endswith('pom.xml'):
            return True
    return False

def filterOutNonMavenProjects(repos: list):
    maven_repos = []
    for repo in repos:
        if isMavenProject(repo):
            print(repo.full_name + ' is OK, stars: ' + str(repo.stargazers_count))
            maven_repos.append(repo)
        else:
            print(repo.full_name + ' is NOT a maven project!, stars: ' + str(repo.stargazers_count))
    return maven_repos

def searchTopNPopularJavaRepos(g: Github, n: int=1000,
                               tokens=MINING_TOKENS):
    repo_objects = []
    # raw_repos = g.search_repositories('language:java', 'stars', 'desc') # buggy
    num_of_pages = int(n/100)
    for i in range(num_of_pages):
        g = Github(tokens[i])
        search_repo_url = "https://api.github.com/search/repositories?" + \
                          "q=language:java" + \
                          "&sort=stars" + \
                          "&order=desc" + \
                          "&per_page=100" + \
                          "&page=" + str(i+1)
        response = requests.get(search_repo_url).content.decode("utf-8")
        #print (response)
        raw_repos = json.loads(response)['items']
        for raw_repo in raw_repos:
            #print (repo['full_name'])
            repo_obj = g.get_repo(raw_repo['full_name'])
            repo_objects.append(repo_obj)
    return repo_objects

def isDependabotPullRequest(pr: PullRequest):
    # Bump sequelize from 4.41.2 to 5.15.1
    pattern = re.compile("Bump .* from .* to .*")
    # print('title: ' + pr.title)
    # print('user: ' + pr.user.login)
    if re.match(pattern, pr.title) and pr.user.login == 'dependabot[bot]':
        return True
    return False

def isAlreadyAnalyzed(repo: Repository, repo_list: list):
    for repo_dict in repo_list:
        if repo_dict['repo'] == repo.full_name:
            return True
    return False

def extractDependabotPullRequestsJSON(maven_repos: list,
                                      existing_json_file: str=None):
    if existing_json_file:
        fr = open(existing_json_file, 'r')
        repo_list = json.load(fr)
        fr.close()
    else:
        repo_list = []
    for repo in maven_repos:
        if isAlreadyAnalyzed(repo, repo_list):
            continue
        print('maven repo: ' + repo.full_name + ' =====')
        repo_dict = collections.OrderedDict({})
        repo_dict['repo'] = repo.full_name
        pr_list = []
        pulls = repo.get_pulls(state='all', sort='created')
        print('num of prs: ' + str(len(list(pulls))))
        for pr in pulls:
            pr_dict = collections.OrderedDict({})
            if isDependabotPullRequest(pr):
                pr_dict['title'] = pr.title
                pr_dict['user'] = pr.user.login
                pr_dict['state'] = pr.state
                pr_dict['url'] = pr.html_url
                pr_dict['merged'] = pr.is_merged()
                pr_list.append(pr_dict)
                print (json.dumps(pr_dict))
        repo_dict['pull_requests'] = pr_list
        repo_list.append(repo_dict)
    return json.dumps(repo_list)

def analyzePullRequests():
    # First create a Github instance:
    g = Github("1c5c30b8033a7a96976633dbbbd0a28282e6aea5")
    print(g.get_rate_limit().core,
          g.get_rate_limit().search,
          g.get_rate_limit().graphql)
    # maven repos
    repos = searchTopNPopularJavaRepos(g, n=1000)
    maven_repos = filterOutNonMavenProjects(repos)
    print('*** ' + str(len(maven_repos)) + ' repos are maven projects ***')
    pr_json = extractDependabotPullRequestsJSON(maven_repos,
                                                existing_json_file=PR_JSON_FILE)
    fw = open(PR_JSON_FILE, 'w')
    fw.write(pr_json)
    fw.close()

if __name__ == '__main__':
    opts = parseArgs(sys.argv[1:])
    if opts.analyze_prs:
        analyzePullRequests()
    elif opts.collect_all_repos:
        collectAllRepos()
