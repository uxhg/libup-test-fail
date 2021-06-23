#!/usr/bin/env python3
import json
import logging
from pathlib import Path
from typing import List, Optional, Dict

from common import ALL_PAIRS_JSON, init_logging

logger = logging.getLogger(__name__)


def collect_library_client_pairs(pairs_json: Path) -> Optional[Dict[str, Dict[str, List[str]]]]:
    """
    :param pairs_json: path to the incompat-pairs-all.json file
    :return: a dict if succeeded
    { library_name: [client1#test1, client1#test2, client2#test1, ...] }
    """
    with open(pairs_json) as allp:
        cli_lib_pairs: List[dict] = json.load(allp)
    if not cli_lib_pairs:
        logger.error(f"Loading pairs from {pairs_json} failed")
        return None
    # we only care clients here, so read pairs into a set to only keep one entry for each client
    libraries: Dict[str, Dict[str, List[str]]] = {}
    for c in cli_lib_pairs:
        lib = c.get("lib")
        client = c.get("client")
        test_name = c.get("test")
        if not (lib and client and test_name):
            logger.warning(f"Skip entry {c}")
            continue
        # client_test = f'{c.get("client")}#{c.get("test")}'
        if lib in libraries:
            if client in libraries[lib]:
                libraries[lib][client].append(test_name)
            else:
                libraries[lib][client] = [test_name]
        else:
            libraries[lib] = {client: [test_name]}
    logger.info(f"In total {len(libraries)} clients to analyze")
    return {k: v for k, v in sorted(libraries.items(), key=lambda x: len(x[1]), reverse=True)}


def save_libraries_compact(data: Dict[str, Dict[str, List[str]]], filename: Path):
    compact = {k: list(v.keys()) for k, v in data.items()}
    with open(filename, 'w') as out_f:
        json.dump(compact, out_f, indent=2)


def save_dict(data: dict, filename: Path):
    with open(filename, 'w') as out_f:
        json.dump(data, out_f, indent=2)


def main():
    libs = collect_library_client_pairs(ALL_PAIRS_JSON)
    save_libraries_compact(libs, Path("libs.json"))


if __name__ == '__main__':
    init_logging()
    main()
