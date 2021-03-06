# coding: utf-8
import argparse
import logging
import sys
from random import seed

from synergine2.log import get_default_logger
from synergine2.config import Config
from synergine2_cocos2d.util import get_map_file_path_from_dir
from synergine2.core import Core
from synergine2.cycle import CycleManager
from synergine2.terminals import TerminalManager

from opencombat.ai.placement import Placement
from opencombat.simulation.base import TileStrategySimulation
from opencombat.simulation.base import TileStrategySubjects
from opencombat.state import StateConstructorBuilder
from opencombat.strategy.troops import TroopConstructorBuilder
from opencombat.terminal.base import CocosTerminal


def main(
    map_dir_path: str,
    seed_value: int=None,
    state_file_path: str=None,
    troops_file_path: str=None,
    state_save_dir: str='.',
    placement_mode: bool = False,
):
    assert not (state_file_path and troops_file_path),\
        'Do not provide troops file when state file given'

    if seed_value is not None:
        seed(seed_value)

    config = Config()
    config.load_yaml('config.yaml')

    # Runtime config
    config['_runtime'] = {}
    config['_runtime']['state_save_dir'] = state_save_dir
    config['_runtime']['placement_mode'] = placement_mode
    config['_runtime']['map_dir_path'] = map_dir_path

    level = logging.getLevelName(config.resolve('global.logging_level', 'ERROR'))
    logger = get_default_logger(level=level)

    map_file_path = get_map_file_path_from_dir(map_dir_path)

    simulation = TileStrategySimulation(config, map_file_path=map_file_path)
    subjects = TileStrategySubjects(simulation=simulation)
    simulation.subjects = subjects

    if state_file_path:
        state_loader_builder = StateConstructorBuilder(config, simulation)
        state_loader = state_loader_builder.get_state_loader()
        state = state_loader.get_state(state_file_path)
        subjects.extend(state.subjects)

    elif troops_file_path:
        troop_loader_builder = TroopConstructorBuilder(config, simulation)
        troop_loader = troop_loader_builder.get_troop_loader()
        placement = Placement(config, simulation)

        troops = troop_loader.get_troop(troops_file_path)
        subjects.extend(troops.subjects)
        placement.place()

    core = Core(
        config=config,
        simulation=simulation,
        cycle_manager=CycleManager(
            config=config,
            simulation=simulation,
        ),
        terminal_manager=TerminalManager(
            config=config,
            terminals=[CocosTerminal(
                config,
                asynchronous=False,
                map_dir_path=map_dir_path,
            )]
        ),
        cycles_per_seconds=1 / config.resolve('core.cycle_duration'),
    )
    core.run()

if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        description='Run a map'
    )
    parser.add_argument('map_dir_path', help='map directory path')
    parser.add_argument('--seed', dest='seed', default=None)
    parser.add_argument('--troops', dest='troops', default=None)
    parser.add_argument('--state', dest='state', default=None)
    parser.add_argument(
        '--state-save-dir',
        dest='state_save_dir',
        default='.',
    )
    parser.add_argument(
        '--placement',
        dest='placement',
        action='store_true',
    )

    args = parser.parse_args()

    if args.troops and args.state:
        print(
            'Cannot load state "{}" because you provide troops file "{}". '
            'You must provide only one of them.'.format(
                args.state,
                args.troops,
            ),
            file=sys.stderr,
        )
        exit(1)

    if args.troops and not args.placement:
        print(
            'Cannot load troops "{}" without activate placement mode.'.format(
                args.state,
            ),
            file=sys.stderr,
        )
        exit(1)

    main(
        args.map_dir_path,
        seed_value=args.seed,
        state_file_path=args.state,
        troops_file_path=args.troops,
        state_save_dir=args.state_save_dir,
        placement_mode=args.placement,
    )
