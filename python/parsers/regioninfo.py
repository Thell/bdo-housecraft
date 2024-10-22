# This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

import kaitaistruct
from kaitaistruct import KaitaiStruct, KaitaiStream, BytesIO
from enum import Enum


if getattr(kaitaistruct, 'API_VERSION', (0, 9)) < (0, 9):
    raise Exception("Incompatible Kaitai Struct Python API: 0.9 or later is required, but you have %s" % (kaitaistruct.__version__))

class Regioninfo(KaitaiStruct):

    class RegionType(Enum):
        minor_town = 0
        main_town = 1
        hunting = 2
        king_war = 3
        lord_war = 4
        castle = 5
        arena = 6
        count = 7

    class VillageSiegeType(Enum):
        sunday = 0
        monday = 1
        tuesday = 2
        wednesday = 3
        thursday = 4
        friday = 5
        saturday = 6
        count = 7

    class RegionAttributeType(Enum):
        normal_land = 0
        beach = 1
        desert = 2
        count = 3
    def __init__(self, _io, _parent=None, _root=None):
        self._io = _io
        self._parent = _parent
        self._root = _root if _root else self
        self._read()

    def _read(self):
        self.pabr = (self._io.read_bytes(4)).decode(u"utf-8")
        self.num_regioninfo = self._io.read_u4le()
        self.regioninfo_table = []
        for i in range(self.num_regioninfo):
            self.regioninfo_table.append(Regioninfo.RegioninfoType(self._io, self, self._root))

        self.string_table = Regioninfo.StringTableType(self._io, self, self._root)

    class RegionSkillType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.is_integrity = self._io.read_u2le()
            self.skill_key = self._io.read_u2le()
            self.apply_rate = self._io.read_s4le()


    class RegioninfoType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.id = self._io.read_u2le()
            self.color_key = Regioninfo.ColorType(self._io, self, self._root)
            self.region_type = KaitaiStream.resolve_enum(Regioninfo.RegionType, self._io.read_u1())
            self.village_siege_type = KaitaiStream.resolve_enum(Regioninfo.VillageSiegeType, self._io.read_u4le())
            self.unknown_enum_type_1_nws = self._io.read_u1()
            self.is_safe_zone = self._io.read_u1()
            self.is_arena_zone = self._io.read_u1()
            self.is_sea = self._io.read_u1()
            self.is_desert = self._io.read_u1()
            self.is_not_dig = self._io.read_u1()
            self.is_ocean = self._io.read_u1()
            self.is_prison = self._io.read_u1()
            self.is_king_war_zone_only_unsure = self._io.read_u1()
            self.is_lord_war_zone_only_unsure = self._io.read_u1()
            self.is_village_war_area = self._io.read_u1()
            self.is_king_or_lord_war_zone = self._io.read_u1()
            self.is_main_town = self._io.read_u1()
            self.is_minor_town = self._io.read_u1()
            self.is_main_or_minor_town = self._io.read_u1()
            self.accessible_area = self._io.read_u1()
            self.is_node_access = self._io.read_u1()
            self.village_tax_level = self._io.read_u1()
            self.unknown_1_nws = self._io.read_u2le()
            self.unknown_2 = self._io.read_u4le()
            self.unknown_3 = self._io.read_u1()
            self.unknown_bool_indicator_1 = self._io.read_u1()
            self.villain_respawn_explore_key = self._io.read_u4le()
            self.villain_respawn_postion = Regioninfo.PositionType(self._io, self, self._root)
            self.is_ancient_dungeon = self._io.read_u1()
            self.is_escape = self._io.read_u1()
            self.is_special_zone = self._io.read_u1()
            self.vehicle_dead_zone = self._io.read_u1()
            self.region_skill_1 = Regioninfo.RegionSkillType(self._io, self, self._root)
            self.region_skill_2 = Regioninfo.RegionSkillType(self._io, self, self._root)
            self.region_skill_3 = Regioninfo.RegionSkillType(self._io, self, self._root)
            self.region_night_skill = Regioninfo.RegionSkillType(self._io, self, self._root)
            self.territory_key = self._io.read_u2le()
            self.index_area_name = self._io.read_u4le()
            self.index_return_position_name = self._io.read_u4le()
            self.affiliated_town_region_key = self._io.read_u2le()
            self.trade_origin_region_key = self._io.read_u2le()
            self.region_group_key = self._io.read_u2le()
            self.immediate_respawn = self._io.read_u1()
            self.waypoint_key = self._io.read_u4le()
            self.explore_key = self._io.read_u4le()
            self.region_attribute_type = KaitaiStream.resolve_enum(Regioninfo.RegionAttributeType, self._io.read_u4le())
            self.center_position = Regioninfo.PositionType(self._io, self, self._root)
            self.return_position = Regioninfo.PositionType(self._io, self, self._root)
            self.valid_return_position_list_len = self._io.read_u4le()
            self.valid_return_position_list = []
            for i in range(self.valid_return_position_list_len):
                self.valid_return_position_list.append(Regioninfo.PositionType(self._io, self, self._root))

            self.exploration_point = self._io.read_u2le()
            self.weather_data = Regioninfo.WeatherDataType(self._io, self, self._root)
            self.warehouse_character_key = self._io.read_u2le()
            self.warehouse_dialog_index = self._io.read_u2le()
            self.npc_worker_character_key = self._io.read_u2le()
            self.npc_worker_dialog_index = self._io.read_u2le()
            self.stable_character_key = self._io.read_u2le()
            self.sable_dialog_index = self._io.read_u2le()
            self.wharf_character_key = self._io.read_u2le()
            self.wharf_dialog_index = self._io.read_u2le()
            self.item_market_character_key = self._io.read_u2le()
            self.item_market_dialog_index = self._io.read_u2le()
            self.delivery_character_key = self._io.read_u2le()
            self.delivery_dialog_index = self._io.read_u2le()
            self.is_free_revival_area = self._io.read_u1()
            self.pc_deliver_region_key_list_len = self._io.read_u4le()
            self.pc_deliver_region_key_list = []
            for i in range(self.pc_deliver_region_key_list_len):
                self.pc_deliver_region_key_list.append(Regioninfo.FieldListType(self._io, self, self._root))

            self.respawn_position_list_len = self._io.read_u4le()
            self.respawn_position_list = []
            for i in range(self.respawn_position_list_len):
                self.respawn_position_list.append(Regioninfo.PositionType(self._io, self, self._root))

            self.unknown_5 = self._io.read_u1()
            self.unknown_6 = self._io.read_u2le()
            self.unknown_7_nws = self._io.read_u2le()
            self.unknown_8_nws = self._io.read_u4le()
            self.unknown_9_nws = self._io.read_u4le()
            self.unknown_10_nws = self._io.read_u4le()
            self.unknown_11_nws = self._io.read_u4le()
            self.unknown_12_nws = self._io.read_u2le()
            self.unknown_13_nws = self._io.read_u1()
            self.world_boss_reward = self._io.read_u1()
            self.escape_point = Regioninfo.PositionType(self._io, self, self._root)
            self.revive_point = Regioninfo.PositionType(self._io, self, self._root)
            self.alignment_padding = self._io.read_u8le()
            self.unknown_14_nws = self._io.read_u4le()
            self.unknown_15_nws = self._io.read_u4le()
            self.unknown_16 = self._io.read_u4le()
            self.is_pit_of_the_undying = self._io.read_u1()
            self.premium_character_possible_region = self._io.read_u1()
            self.unknown_17nodewars_siege_related = self._io.read_u4le()
            self.unknown_18 = self._io.read_u2le()
            self.unknown_19_nws = self._io.read_u4le()
            self.unknown_20_nws = self._io.read_u4le()
            self.unknown_21nodewars_siege_related = self._io.read_u4le()
            self.unknown_22nodewars_siege_related = self._io.read_u4le()
            self.unknown_23nodewars_siege_related = self._io.read_u4le()
            self.unknown_24_nws = self._io.read_u4le()
            self.unknown_25_nws = self._io.read_u4le()
            self.unknown_26_nws = self._io.read_u4le()
            self.uknown_arena_something = self._io.read_u1()
            self.unknown_27 = self._io.read_bytes(12)
            self.unknown_28_siege_related = self._io.read_bytes(12)
            self.unknown_29_nws = self._io.read_u2le()
            self.is_siege_challenge = self._io.read_u1()
            self.unknown_30_siege_related = self._io.read_u8le()
            self.unknown_31_siege_related = self._io.read_u8le()
            self.unknown_32_nws = self._io.read_u8le()
            self.unknown_33_nws = self._io.read_u8le()

        @property
        def area_name(self):
            if hasattr(self, '_m_area_name'):
                return self._m_area_name

            self._m_area_name = self._root.string_table.strings[self.index_area_name].string_utf16
            return getattr(self, '_m_area_name', None)


    class WeatherDataType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.is_initialize_weather = self._io.read_u1()
            self.unknown_3 = self._io.read_u2le()
            self.unknown_4 = self._io.read_u1()
            self.temperature = self._io.read_f4le()
            self.humility = self._io.read_f4le()
            self.oil_rate = self._io.read_f4le()
            self.water_rate = self._io.read_f4le()
            self.rain_amount = self._io.read_f4le()
            self.rain_tick_ready = self._io.read_f4le()
            self.rain_tick_keep = self._io.read_f4le()
            self.select_weather_time = self._io.read_s4le()


    class FieldListType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.field = self._io.read_u2le()


    class StringTableType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.num_strings = self._io.read_u4le()
            self.strings = []
            for i in range(self.num_strings):
                self.strings.append(Regioninfo.StringBlockType(self._io, self, self._root))



    class PositionType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.x = self._io.read_f4le()
            self.y = self._io.read_f4le()
            self.z = self._io.read_f4le()


    class ColorType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.b = self._io.read_u1()
            self.g = self._io.read_u1()
            self.r = self._io.read_u1()
            self.a = self._io.read_u1()


    class StringBlockType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.is_utf16_string = self._io.read_u1()
            self.string_length = self._io.read_u4le()
            if self.is_utf16_string == 0:
                self.string_utf8 = (self._io.read_bytes(self.string_length)).decode(u"utf-8")

            if self.is_utf16_string == 1:
                self.string_utf16 = (self._io.read_bytes(self.string_length)).decode(u"utf-16le")




