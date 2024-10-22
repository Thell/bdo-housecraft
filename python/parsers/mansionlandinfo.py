# This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

import kaitaistruct
from kaitaistruct import KaitaiStruct, KaitaiStream, BytesIO


if getattr(kaitaistruct, 'API_VERSION', (0, 9)) < (0, 9):
    raise Exception("Incompatible Kaitai Struct Python API: 0.9 or later is required, but you have %s" % (kaitaistruct.__version__))

class Mansionlandinfo(KaitaiStruct):
    def __init__(self, _io, _parent=None, _root=None):
        self._io = _io
        self._parent = _parent
        self._root = _root if _root else self
        self._read()

    def _read(self):
        self.header = (self._io.read_bytes(4)).decode(u"UTF-8")
        self.num_offset_table = self._io.read_u4le()
        self.mansionlandinfo_table = []
        for i in range(self.num_offset_table):
            self.mansionlandinfo_table.append(Mansionlandinfo.MansionlandinfoType(self._io, self, self._root))


    class MansionlandinfoType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.land_character_key = self._io.read_u2le()
            self.building_character_key = self._io.read_u2le()
            self.need_point = self._io.read_u2le()
            self.region = self._io.read_u2le()
            self.sector_start_position = []
            for i in range(3):
                self.sector_start_position.append(self._io.read_s4le())

            self.sector_end_position = []
            for i in range(3):
                self.sector_end_position.append(self._io.read_s4le())

            self.unknown_sector_position_1 = []
            for i in range(3):
                self.unknown_sector_position_1.append(self._io.read_s4le())

            self.unknown_sector_position_2 = []
            for i in range(3):
                self.unknown_sector_position_2.append(self._io.read_s4le())

            self.condition = []
            for i in range(2):
                self.condition.append(self._io.read_u2le())

            self.day_pay = self._io.read_u8le()
            self.unknown_1 = self._io.read_f4le()
            self.unknown_2 = self._io.read_f4le()
            self.unknown_3 = self._io.read_f4le()
            self.mansion_world_map_icon_type = self._io.read_u1()
            self.guild_mansion_teleport_position = Mansionlandinfo.PositionType(self._io, self, self._root)


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



