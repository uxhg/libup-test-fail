#!/usr/bin/env python3
import json
import logging
from pathlib import Path
from typing import List, Optional, Dict

from common import ALL_PAIRS_JSON

logger = logging.getLogger(__name__)


def collect_library_client_pairs(pairs_json: Path) -> Optional[dict]:
    with open(pairs_json) as allp:
        cli_lib_pairs: List[dict] = json.load(allp)
    if not cli_lib_pairs:
        logger.error(f"Loading pairs from {pairs_json} failed")
        return None
    # we only care clients here, so read pairs into a set to only keep one entry for each client
    libraries: Dict[str, List[str]] = {}
    for c in cli_lib_pairs:
        lib = c["lib"]
        client_test = f'{c.get("client")}#{c.get("test")}'
        if lib in libraries:
            libraries[lib].append(client_test)
        else:
            libraries[lib] = [client_test]
    logger.info(f"In total {len(libraries)} clients to analyze")


def main():
    collect_library_client_pairs(ALL_PAIRS_JSON)


if __name__ == '__main__':
    main()
