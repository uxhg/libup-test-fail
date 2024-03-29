#!/usr/bin/env python3
import argparse
import json
import logging
from pathlib import Path
from pprint import pprint
from typing import List, Dict

from common import init_logging, ALL_CLIENTS_JSON

logger = logging.getLogger(__name__)


def main():
    args = handle_args()
    get_client_info(args.client)


def read_all_clients(data_file) -> Dict[str, dict]:
    with open(data_file) as allc:
        clients: List[dict] = json.load(allc)
        client_dict = {c["name"]: {k: c[k] for k in c.keys() - "name"} for c in clients}
        return client_dict


def get_client_info(cli: str, data_file: Path = ALL_CLIENTS_JSON) -> dict:
    client_dict = read_all_clients(data_file)
    cli_d = client_dict.get(cli)
    if cli_d:
        pprint(cli_d)
        return cli_d
        # url = cli_d["url"]
        # sha = cli_d["sha"]
        # new_dir: Path = LOC_REPO / cli
        # Path.mkdir(new_dir)
        # repo = clone(url, new_dir)
        # checkout(repo, sha)
    else:
        logger.error(f"Client [{cli}] is not found.")


def handle_args():
    parser = argparse.ArgumentParser(description='Find client info in json data.')
    parser.add_argument("client", metavar="CLIENT", type=str, help="client name")
    args = parser.parse_args()
    init_logging()
    logger.debug(args)
    return args


if __name__ == '__main__':
    main()
