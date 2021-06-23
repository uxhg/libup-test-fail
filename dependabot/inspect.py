#!/usr/bin/python3

import os
import sys
import json
from typing import Set, Dict, List
from pathlib import Path

THIS_DIR = Path(os.path.dirname(os.path.realpath(__file__)))
ALL_CLIENTS_JSON: Path = (THIS_DIR / "../depgraph/data/external/all-clients.json").resolve()


def how_many(json_data: dict) -> int:
    items: list = json_data.get("items")
    if items:
        return len(items)
    else:
        return 0

def get_repo_set(json_data: dict) -> Set[str]:
    return {x.get("repository_url").rstrip("/").rsplit("/", maxsplit=1)[1] for x in json_data["items"]}
    

def read_all_clients(data_file) -> Dict[str, dict]:
    with open(data_file) as allc:
        clients: List[dict] = json.load(allc)
        client_dict = {c["name"]: {k: c[k] for k in c.keys() - "name"} for c in clients}
        return client_dict

def find_intersect(found_repos: Set[str], all_clients: Dict[str, dict]) -> set:
    all_clients_names = all_clients.keys()
    return all_clients_names & found_repos


def main():
    input_file = sys.argv[1]
    with open(input_file, 'r') as jf:
        data: dict = json.load(jf)
        print(how_many(data))
        repos = get_repo_set(data) 
        print(repos)
        print(find_intersect(repos, read_all_clients(ALL_CLIENTS_JSON)))


if __name__ == '__main__':
    main()


