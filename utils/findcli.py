#!/usr/bin/env python3
import json
import os
from typing import List
import logging
import argparse
import sys
from pprint import pprint
from pathlib import Path

logger = logging.getLogger(__name__)

THIS_DIR = Path(os.path.dirname(os.path.realpath(__file__)))
ALL_CLIENTS_JSON =  THIS_DIR / "../../libpairs/all-clients.json"
LOC_REPO = THIS_DIR / "../../libpairs/cases"


def main():
    args = handle_args()
    get_client_info(args.client)


def get_client_info(cli: str):
    with open(ALL_CLIENTS_JSON) as allc:
        clients: List[dict] = json.load(allc)
        client_dict = {c["name"]: {k: c[k] for k in c.keys() - "name"} for c in clients}
        cli_d = client_dict.get(cli)
        if cli_d:
            pprint(cli_d)
            # url = cli_d["url"]
            # sha = cli_d["sha"]
            # new_dir: Path = LOC_REPO / cli
            # Path.mkdir(new_dir)
            # repo = clone(url, new_dir)
            # checkout(repo, sha)
        else:
            logger.error(f"Client [{cli}] is not found.")

#def clone(url: str, local: Path):
#    logger.info(f"Clone {url} into {local}.")
#    repo: git.Repo = git.Git(local).clone(url)
#    return repo
#
#
#def checkout(repo: git.Repo, sha: str):
#    logger.info(f"Checkout {repo.working_dir} -> {sha}.")
#    repo.git.checkout(sha)


def handle_args():
    parser = argparse.ArgumentParser(description='Find client info in json data.')
    parser.add_argument("client", metavar="CLIENT", type=str, help="client name")
    parser.add_argument('-l', metavar='loglevel', type=str, required=False, help='logging level, default WARNING')
    args = parser.parse_args()
    init_logging(args.l)
    logger.debug(args)
    return args


class ColorFormatter(logging.Formatter):
    COLORS = {
        logging.DEBUG: "\033[96m",
        logging.INFO: "\033[92m",
        logging.WARNING: "\033[93m",
        logging.ERROR: "\033[91m",
        logging.CRITICAL: "\033[01;91m\033[47m",  # bold red on white background
        'RESET': "\033[0m"
    }

    def format(self, record):
        color = self.COLORS[record.levelno]
        color_reset = self.COLORS["RESET"]
        self.datefmt = "%m-%d %H:%M:%S"
        self._style._fmt = color + '[%(asctime)s] [%(levelname)8s] ' + color_reset + '%(message)s'
        return super().format(record)


def init_logging(log_level="warning"):
    root_logger = logging.getLogger()
    if log_level is None:
        log_level = "warning"
    numeric_level = getattr(logging, log_level.upper(), None)
    if not isinstance(numeric_level, int):
        raise ValueError('Invalid log level: %s' % log_level)
    root_logger.setLevel(numeric_level)
    handler = logging.StreamHandler(sys.stderr)
    handler.setFormatter(ColorFormatter())
    root_logger.addHandler(handler)


if __name__ == '__main__':
    main()
