#!/usr/bin/env python3

import argparse
import json
import logging
import shlex
import shutil
import subprocess
from pathlib import Path
from typing import List
from datetime import datetime

from clone_co import clone_co
from common import ClientAtVer, init_logging, ALL_PAIRS_JSON, add_suffix, rm_suffix

logger = logging.getLogger(__name__)

EXP_PATH = Path("~/.local/tmp/exp").expanduser()
TOOL_RUN_PATH = Path("~/Projects/lib-conflict/libup-test-fail/depgraph/scripts").expanduser()


def main():
    args = handle_args()
    run_all(ALL_PAIRS_JSON)


def run_all(pairs_json: Path):
    with open(pairs_json) as allp:
        cli_lib_pairs: List[dict] = json.load(allp)
    # we only care clients here, so read pairs into a set to only keep one entry for each client
    clients = {c["client"]: {k: c[k] for k in ["url", "sha"]} for c in cli_lib_pairs}
    checked = set()
    for c, c_data in clients.items():
        cli = ClientAtVer(name=c, url=c_data["url"], sha1=c_data["sha"])
        if single_client(cli):
            checked.add(c)
        else:
            logger.warning(f"Tool running on {cli.name} returned false")
    with open(EXP_PATH/"stat", 'a') as stat_file:
        stat_file.write(f"{datetime.now().isoformat()}\n")
        stat_file.write(f"{','.join(checked)}\n")


def single_client(cli: ClientAtVer, loc_path: Path = EXP_PATH) -> bool:
    """
    Run tool on one client
    :param cli:
    :param loc_path:
    :return:
    """
    clean_copy: Path = loc_path / add_suffix(cli.name)
    if not clean_copy.resolve().exists():  # no local copy, clone from remote
        logger.info(f"Clone to {clean_copy}")
        r, _ = clone_co(cli, loc_path)
    run_copy = rm_suffix(str(clean_copy))
    if Path(run_copy).exists():
        logger.warning(f"{run_copy} exists, skip {cli.name}")
        return False
    logger.info(f"Copy to {run_copy}")
    shutil.copytree(clean_copy, run_copy)

    all_suc = False

    mods: list = get_module_list(Path(run_copy))
    if len(mods) > 1:
        logger.info("Multiple modules")
        # remove root repo (parent artifact)
        mods.sort(key=len, reverse=True)
        mods.pop()  # remove last, i.e., shortest, i.e, root path
        for m in mods:
            suc = single_mod(Path(m))
            all_suc = all_suc and suc
        return all_suc
    else:
        return single_mod(Path(run_copy))


def single_mod(mod_path: Path) -> bool:
    # run tool
    args: list = shlex.split(f"./run.sh {mod_path}")
    completed: subprocess.CompletedProcess = subprocess.run(args, cwd=TOOL_RUN_PATH, shell=False, capture_output=True)
    try:
        completed.check_returncode()
    except subprocess.CalledProcessError:
        return False
    return True


def get_module_list(root_path: Path) -> List[Path]:
    args: list = shlex.split('mvn exec:exec -Dexec.executable="pwd" -q')
    completed: subprocess.CompletedProcess = subprocess.run(args, cwd=root_path, shell=False, capture_output=True)
    try:
        completed.check_returncode()
    except subprocess.CalledProcessError:
        return []
    else:
        ret = [y for x in completed.stdout.decode().split("\n") if len(y := x.rstrip("\n").strip()) > 0]
        logger.info(ret)
        return ret


def handle_args():
    parser = argparse.ArgumentParser(description='Run all cases with run.sh in depgraph')
    parser.add_argument('-l', metavar='loglevel', type=str, required=False, help='logging level, default WARNING')
    args = parser.parse_args()
    init_logging(args.l)
    logger.debug(args)
    return args


if __name__ == '__main__':
    main()
