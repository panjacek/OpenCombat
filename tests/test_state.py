# coding: utf-8
from collections import OrderedDict
import time

import pytest
from synergine2.config import Config
from synergine2_cocos2d.const import SELECTION_COLOR_RGB

from opencombat.exception import StateLoadError
from opencombat.simulation.base import TileStrategySimulation
from opencombat.simulation.base import TileStrategySubjects
from opencombat.simulation.subject import ManSubject
from opencombat.state import StateConstructorBuilder, StateDumper
from opencombat.state import StateLoader
from opencombat.const import FLAG
from opencombat.const import SIDE
from opencombat.const import FLAG_DE
from opencombat.const import DE_COLOR
from opencombat.const import USSR_COLOR
from opencombat.const import FLAG_USSR
from opencombat.const import SIDE_ALLIES
from opencombat.const import SIDE_AXIS


class MyStateLoader(StateLoader):
    pass


@pytest.fixture
def state_loader(config, simulation):
    return StateLoader(config, simulation)


@pytest.fixture
def simulation_for_dump(config) -> TileStrategySimulation:
    simulation = TileStrategySimulation(
        config,
        'tests/fixtures/map_a/map_a.tmx',
    )
    subjects = TileStrategySubjects(simulation=simulation)
    simulation.subjects = subjects

    man1 = ManSubject(config, simulation)
    man1.position = (10, 11)
    man1.direction = 42
    man1.properties = OrderedDict([
        (SELECTION_COLOR_RGB, DE_COLOR),
        (FLAG, FLAG_DE),
        (SIDE, SIDE_AXIS),
    ])

    man2 = ManSubject(config, simulation)
    man2.position = (16, 8)
    man2.direction = 197
    man2.properties = OrderedDict([
        (SELECTION_COLOR_RGB, USSR_COLOR),
        (FLAG, FLAG_USSR),
        (SIDE, SIDE_ALLIES),
    ])

    subjects.append(man1)
    subjects.append(man2)

    return simulation


def test_state_loader_builder__ok__nominal_case(
    simulation,
):
    config = Config({
        'global': {
            'state_loader': 'tests.test_state.MyStateLoader',
        }
    })
    builder = StateConstructorBuilder(config, simulation)
    state_loader = builder.get_state_loader()
    assert type(state_loader) == MyStateLoader


def test_state_loader__ok__load_state(
    state_loader,
):
    assert state_loader.get_state('tests/fixtures/state_ok.xml')


def test_state_loader__error__state_empty(
    state_loader,
):
    with pytest.raises(StateLoadError):
        assert state_loader.get_state('tests/fixtures/state_empty.xml')


def test_state_loader__error__state_invalid(
    state_loader,
):
    with pytest.raises(StateLoadError):
        assert state_loader.get_state(
            'tests/fixtures/state_error_schema.xml',
        )


def test_state__ok__subjects(
    state_loader,
):
    state = state_loader.get_state('tests/fixtures/state_ok.xml')

    assert 2 == len(state.subjects)
    assert isinstance(state.subjects[0], ManSubject)
    assert isinstance(state.subjects[1], ManSubject)

    assert (1, 1) == state.subjects[0].position
    assert (10, 10) == state.subjects[1].position
    assert 90.0 == state.subjects[0].direction
    assert 270.0 == state.subjects[1].direction

    assert 'COMBAT_MODE_DEFEND' == state.subjects[0].combat_mode
    assert 'COMBAT_MODE_HIDE' == state.subjects[1].combat_mode

    assert {
               'SELECTION_COLOR_RGB': (204, 0, 0),
               'FLAG': 'FLAG_USSR',
               'SIDE': 'ALLIES',
           } == state.subjects[0].properties
    assert {
               'SELECTION_COLOR_RGB': (0, 81, 211),
               'FLAG': 'FLAG_DE',
               'SIDE': 'AXIS',
           } == state.subjects[1].properties


def test_state__ok__dump(
    config: Config,
    simulation_for_dump: TileStrategySimulation,
):
    state_dumper = StateDumper(config, simulation_for_dump)
    state_xml_str = state_dumper.get_state_dump()
    assert """<?xml version="1.0" ?>
<state type="before_battle">
    <map>
        <name>tests/fixtures/map_a/map_a.tmx</name>
    </map>
    <subjects>
        <subject>
            <type>opencombat.simulation.subject.ManSubject</type>
            <position>10,11</position>
            <direction>42</direction>
            <combat_mode>COMBAT_MODE_DEFEND</combat_mode>
            <properties>
                <item>
                    <key>SELECTION_COLOR_RGB</key>
                    <value>0,81,211</value>
                </item>
                <item>
                    <key>FLAG</key>
                    <value>FLAG_DE</value>
                </item>
                <item>
                    <key>SIDE</key>
                    <value>SIDE_AXIS</value>
                </item>
            </properties>
        </subject>
        <subject>
            <type>opencombat.simulation.subject.ManSubject</type>
            <position>16,8</position>
            <direction>197</direction>
            <combat_mode>COMBAT_MODE_DEFEND</combat_mode>
            <properties>
                <item>
                    <key>SELECTION_COLOR_RGB</key>
                    <value>204,0,0</value>
                </item>
                <item>
                    <key>FLAG</key>
                    <value>FLAG_USSR</value>
                </item>
                <item>
                    <key>SIDE</key>
                    <value>SIDE_ALLIES</value>
                </item>
            </properties>
        </subject>
    </subjects>
</state>""" == state_xml_str

def test_state__ok__dump_and_load(
    config: Config,
    simulation_for_dump: TileStrategySimulation,
    state_loader,
):
    state_dumper = StateDumper(config, simulation_for_dump)
    state_xml_str = state_dumper.get_state_dump()

    tmp_file_name = '/tmp/{}.xml'.format(time.time())
    with open(tmp_file_name, 'w+') as file:
        file.write(state_xml_str)

    state_loader.get_state(tmp_file_name)
