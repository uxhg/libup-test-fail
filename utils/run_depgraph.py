#!/usr/bin/env python3

import argparse
import json
import logging
import os
import shlex
import shutil
import subprocess
from datetime import datetime
from pathlib import Path
from typing import List, Set, Optional

from clone_co import clone_co
from common import ClientAtVer, init_logging, ALL_PAIRS_JSON, ONE_PAIR_JSON, add_suffix, rm_suffix

logger = logging.getLogger(__name__)

EXP_PATH = Path("~/.local/tmp/exp").expanduser()
TOOL_RUN_PATH = Path("~/Projects/lib-conflict/libup-test-fail/depgraph/").expanduser()
DOT_OUT_PATH = TOOL_RUN_PATH / "../output-new/dot"
HUMAN_OUT_PATH = TOOL_RUN_PATH / "../output-new/human"
DL_PROG_PATH = TOOL_RUN_PATH / "datalog/simple-dataflow.dl"


def main():
    args = handle_args()
    run_all(ALL_PAIRS_JSON)


def run_all(pairs_json: Path):
    with open(pairs_json) as allp:
        cli_lib_pairs: List[dict] = json.load(allp)
    # we only care clients here, so read pairs into a set to only keep one entry for each client
    clients = {c["client"]: {k: c[k] for k in ["url", "sha"]} for c in cli_lib_pairs}
    logger.info(f"In total {len(clients)} clients to analyze")
    checked = set()
    for c, c_data in clients.items():
        # if c in EXIST:
        #     logger.info(f"Skip existing {c}")
        #     continue
        cli = ClientAtVer(name=c, url=c_data["url"], sha1=c_data["sha"])
        if single_client(cli):
            checked.add(c)
        else:
            logger.warning(f"Tool running on {cli.name} returned false")
    with open(EXP_PATH / "stat", 'a') as stat_file:
        stat_file.write(f"{datetime.now().isoformat()}\n")
        stat_file.write(f"{','.join(checked)}\n")


def run_analysis(cli: ClientAtVer, mod_path: Path) -> bool:
    """
    Run analysis and record output
    :param cli:
    :return:
    """
    logger.info(f"Run analysis on {cli.name}: {mod_path.name}")
    dot_out_per_mod = DOT_OUT_PATH / cli.name / f"{mod_path.name}.dot"
    human_out_per_mod = HUMAN_OUT_PATH / cli.name / f"{mod_path.name}.flow"
    for x in (dot_out_per_mod.parent, human_out_per_mod.parent):
        if not x.exists():
            os.makedirs(x)
    args: list = shlex.split(f"target/release/analyze -i {mod_path} --dl {DL_PROG_PATH} -o {dot_out_per_mod}")
    completed: subprocess.CompletedProcess = subprocess.run(args, cwd=TOOL_RUN_PATH, shell=False, capture_output=True)
    try:
        completed.check_returncode()
        with open(human_out_per_mod, 'w') as hf:
            hf.write(completed.stdout.decode())
    except subprocess.CalledProcessError:
        return False
    return True


def single_client(cli: ClientAtVer, loc_path: Path = EXP_PATH, reuse_facts: bool = True) -> Optional[Set]:
    """
    Run tool on one client
    :param cli:
    :param loc_path:
    :param reuse_facts: check if .facts/ exist, and do not reproduce if exist
    :return: set of failed modules or None if all succeeded
    """
    clean_copy: Path = loc_path / add_suffix(cli.name)
    if not clean_copy.resolve().exists():  # no local copy, clone from remote
        logger.info(f"Clone to {clean_copy}")
        r, _ = clone_co(cli, loc_path)
    run_copy: Path = Path(rm_suffix(str(clean_copy)))
    if run_copy.exists():
        logger.warning(f"[{cli.name}] {run_copy} exists")
        # return False
    else:
        logger.info(f"Copy to {run_copy}")
        shutil.copytree(clean_copy, run_copy)

    mods: list = get_module_list(run_copy)
    if len(mods) > 1:
        logger.info("Multiple modules")
        # remove root repo (parent artifact)
        mods.sort(key=len, reverse=True)
        mods.pop()  # remove last, i.e., shortest, i.e, root path
        failed_mods = set()
        for m in mods:
            mod_path = Path(m)
            facts_dir: Path = mod_path / ".facts"
            if reuse_facts and Path(facts_dir).exists():
                logger.info(f"{facts_dir} exists, reuse existing facts for {cli.name}")
                suc = run_analysis(cli, mod_path)
                if not suc:
                    failed_mods.add(mod_path.name)
            else:
                logger.info(f"Currently we do analyses only so skip all mods without existing facts")
                continue
                # following code is unreachable
                suc = single_mod(mod_path)
                if not suc:
                    logger.warning(f"Facts collection on {cli.name}: {mod_path} failed")
                    failed_mods.add(mod_path.name)
                suc = run_analysis(cli, mod_path)
                if not suc:
                    failed_mods.add(mod_path.name)
        if len(failed_mods) > 0:
            return failed_mods
        else:
            return None
    else:
        # if single_mod(run_copy):
        if run_analysis(cli, run_copy):
            return None
        else:
            return {run_copy}


def single_mod(mod_path: Path) -> bool:
    # run tool
    args: list = shlex.split(f"./scripts/run.sh {mod_path}")
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
        logger.info(f"The list of modules: {ret}")
        return ret


def write_results_toml(out_file: Path, cli: ClientAtVer):
    pass


def handle_args():
    parser = argparse.ArgumentParser(description='Run all cases with run.sh in depgraph')
    parser.add_argument('-l', metavar='loglevel', type=str, required=False, help='logging level, default WARNING')
    args = parser.parse_args()
    init_logging(args.l)
    logger.debug(args)
    return args


if __name__ == '__main__':
    main()
