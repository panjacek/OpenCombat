# coding: utf-8
import os
import typing

import time

import pyglet
from PIL import Image
from synergine2.config import Config
from synergine2.simulation import Subject
from synergine2_cocos2d.actor import Actor
from synergine2_xyz.exception import UnknownAnimationIndex

from opencombat.exception import UnknownWeapon
from opencombat.exception import WrongMode
from opencombat.exception import UnknownFiringAnimation
from opencombat.game.animation import ANIMATION_CRAWL
from opencombat.game.animation import ANIMATION_WALK
from opencombat.game.const import MODE_MAN_STAND_UP
from opencombat.game.const import MODE_MAN_CRAWLING
from opencombat.game.image import TileImageCacheManager
from opencombat.game.weapon import RIFFLE
from opencombat.game.weapon import WeaponImageApplier
from opencombat.user_action import UserAction

if typing.TYPE_CHECKING:
    from opencombat.game.fire import GuiFiringEvent


MODE_DEFAULT = 'MODE_DEFAULT'


class BaseActor(Actor):
    position_matching = {
        ANIMATION_WALK: MODE_MAN_STAND_UP,
        ANIMATION_CRAWL: MODE_MAN_CRAWLING,
    }
    mode_image_paths = {
        MODE_DEFAULT: 'unknown.png',
    }
    modes = [
        MODE_DEFAULT,
    ]
    weapons_firing_image_scheme = {}
    weapon_image_scheme = {}
    move_for_gui_actions = {}

    def __init__(
        self,
        image_path: str,
        config: Config,
        subject: Subject,
    ) -> None:
        self._mode = MODE_MAN_STAND_UP
        self.weapon_image_applier = WeaponImageApplier(config, self)
        self.firing_texture_cache = {}  # type: typing.Dict[str, typing.Dict[str, typing.List[pyglet.image.AbstractImage]]  # nopep8
        super().__init__(image_path, subject=subject, config=config)

        # Firing
        self.last_firing_time = 0
        self.firing_change_image_gap = 0.05  # seconds

    def get_image_cache_manager(self) -> TileImageCacheManager:
        return TileImageCacheManager(self, self.config)

    def get_default_mode(self) -> str:
        return MODE_DEFAULT

    def get_mode_image_path(self, mode: str) -> str:
        return self.mode_image_paths[mode]

    def get_modes(self) -> typing.List[str]:
        return self.modes

    @property
    def mode(self) -> str:
        return self._mode

    @mode.setter
    def mode(self, value) -> None:
        if value not in self.get_modes():
            raise WrongMode('Actor "{}" has no mode "{}" ({})'.format(
                self.__class__.__name__,
                value,
                ', '.join(self.get_modes()),
            ))

        self._mode = value

    def get_mode_for_gui_action(self, gui_action: str) -> str:
        try:
            return self.move_for_gui_actions[gui_action]
        except KeyError:
            return self.get_default_mode()

    @property
    def weapons(self) -> typing.List[str]:
        return []

    def build_textures_cache(self) -> None:
        super().build_textures_cache()
        self.build_firing_texture_cache()

    def get_default_appliable_images(self) -> typing.List[Image.Image]:
        if not self.weapons:
            return []

        return [
            self.weapon_image_applier.get_image_for_weapon(
                self.mode,
                # TODO BS 2018-02-08: Change this when weapon management enhanced
                self.weapons[0],
            )
        ]

    def can_rotate_instant(self) -> bool:
        return True

    def get_animation_appliable_images(
        self,
        animation_name: str,
        animation_position: int,
    ) -> typing.List[Image.Image]:
        if not self.weapons:
            return []

        position = self.position_matching[animation_name]

        try:
            return [
                self.weapon_image_applier.get_animation_image_for_weapon(
                    position,
                    self.weapons[0],
                    animation_position,
                )
            ]
        except UnknownWeapon:
            return []

    def build_firing_texture_cache(self) -> None:
        cache_dir = self.config.resolve('global.cache_dir_path')
        for mode in self.get_modes():
            for weapon in self.weapons:
                firing_images = self.image_cache_manager.firing_cache.get_list(
                    mode,
                    weapon,
                )
                for i, firing_image in enumerate(firing_images):
                    image_name = '{}_firing_{}_{}_{}.png'.format(
                        str(self.subject.id),
                        mode,
                        weapon,
                        i,
                    )
                    cache_image_path = os.path.join(cache_dir, image_name)
                    firing_image.save(cache_image_path)

                    self.firing_texture_cache\
                        .setdefault(mode, {})\
                        .setdefault(weapon, [])\
                        .append(pyglet.image.load(cache_image_path))

    def firing(self, firing: 'GuiFiringEvent') -> None:
        # FIXME: move some code ?
        now = time.time()
        if now - self.last_firing_time >= self.firing_change_image_gap:
            self.last_firing_time = now
            firing.increment_animation_index()

            try:
                texture = self.firing_texture_cache\
                    [self.mode]\
                    [firing.weapon]\
                    [firing.animation_index]
            except KeyError:
                self.logger.error(
                    'No firing animation for actor {}({}) for mode "{}"'
                    ' and weapon "{}"'.format(
                        self.__class__.__name__,
                        str(self.subject.id),
                        self.mode,
                        firing.weapon,
                    )
                )
                return  # There is no firing animation defined
            except IndexError:
                texture = self.firing_texture_cache\
                    [self.mode]\
                    [firing.weapon]\
                    [0]
                firing.reset_animation_index()

            self.update_image(texture)


class Man(BaseActor):
    animation_image_paths = {
        ANIMATION_WALK: [
            'actors/man.png',
            'actors/man_w1.png',
            'actors/man_w2.png',
            'actors/man_w3.png',
            'actors/man_w4.png',
            'actors/man_w5.png',
            'actors/man_w6.png',
            'actors/man_w7.png',
        ],
        ANIMATION_CRAWL: [
            'actors/man_c1.png',
            'actors/man_c2.png',
            'actors/man_c3.png',
            'actors/man_c4.png',
        ]
    }
    modes = [
        MODE_MAN_STAND_UP,
        MODE_MAN_CRAWLING,
    ]
    mode_image_paths = {
        MODE_MAN_STAND_UP: 'actors/man.png',
        MODE_MAN_CRAWLING: 'actors/man_c1.png',
    }
    weapon_image_scheme = {
        MODE_MAN_STAND_UP: {
            RIFFLE: [
                'actors/man_weap1.png'
            ],
        },
        MODE_MAN_CRAWLING: {
            RIFFLE: [
                'actors/man_c1_weap1.png',
                'actors/man_c2_weap1.png',
                'actors/man_c3_weap1.png',
                'actors/man_c4_weap1.png',
            ],

        }
    }
    weapons_firing_image_scheme = {
        MODE_MAN_STAND_UP: {
            RIFFLE: [
                'actors/man_weap1_firing1.png',
                'actors/man_weap1_firing2.png',
                'actors/man_weap1_firing3.png',
            ],
        },
        MODE_MAN_CRAWLING: {
            RIFFLE: [
                'actors/man_weap1_firing1.png',
                'actors/man_weap1_firing2.png',
                'actors/man_weap1_firing3.png',
            ]
        }
    }
    move_for_gui_actions = {
        UserAction.ORDER_MOVE: MODE_MAN_STAND_UP,
        UserAction.ORDER_MOVE_FAST: MODE_MAN_STAND_UP,
        UserAction.ORDER_MOVE_CRAWL: MODE_MAN_CRAWLING,
    }

    def __init__(
        self,
        config: Config,
        subject: Subject,
    ) -> None:
        super().__init__('actors/man.png', subject=subject, config=config)

    @property
    def weapons(self) -> typing.List[str]:
        # TODO BS 2018-01-26: Will be managed by complex part of code
        return [RIFFLE]

    def get_default_mode(self) -> str:
        return MODE_MAN_STAND_UP


class HeavyVehicle(BaseActor):
    animation_image_paths = {
        ANIMATION_WALK: [
            'actors/tank1.png',
        ],
        ANIMATION_CRAWL: [
            'actors/tank1.png',
        ]
    }
    mode_image_paths = {
        MODE_DEFAULT: 'actors/tank1.png',
    }

    def __init__(
        self,
        config: Config,
        subject: Subject,
    ) -> None:
        super().__init__('actors/tank1.png', subject=subject, config=config)

    @property
    def weapons(self) -> typing.List[str]:
        # TODO BS 2018-01-26: Will be managed by complex part of code
        return [RIFFLE]

    def can_rotate_instant(self) -> bool:
        return False
