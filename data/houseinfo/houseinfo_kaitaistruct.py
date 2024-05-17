# This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

import kaitaistruct
from kaitaistruct import KaitaiStruct, KaitaiStream, BytesIO


if getattr(kaitaistruct, 'API_VERSION', (0, 9)) < (0, 9):
    raise Exception("Incompatible Kaitai Struct Python API: 0.9 or later is required, but you have %s" % (kaitaistruct.__version__))

class Houseinfo(KaitaiStruct):
    def __init__(self, _io, _parent=None, _root=None):
        self._io = _io
        self._parent = _parent
        self._root = _root if _root else self
        self._read()

    def _read(self):
        self.header = (self._io.read_bytes(4)).decode(u"UTF-8")
        self.num_entries = self._io.read_u4le()
        self.house_info = []
        for i in range(self.num_entries):
            self.house_info.append(Houseinfo.HouseinfoType(self._io, self, self._root))


    class CraftListType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.item_craft_index = self._io.read_u4le()
            self.house_level = self._io.read_u4le()


    class NeedHouseKeyType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.need_house_key = self._io.read_u2le()


    class HouseinfoType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.need_explore_point = self._io.read_u2le()
            self.affiliated_warehouse = self._io.read_u2le()
            self.parent_node = self._io.read_u2le()
            self.unk1 = self._io.read_u2le()
            self.character_key = self._io.read_u2le()
            self.unk2 = []
            for i in range(2):
                self.unk2.append(self._io.read_u1())

            self.house_group = self._io.read_u4le()
            self.house_floor = self._io.read_u4le()
            self.has_need_house_key = self._io.read_u4le()
            self.unk3 = self._io.read_u4le()
            if self.has_need_house_key == 1:
                self.need_house_key = Houseinfo.NeedHouseKeyType(self._io, self, self._root)

            self.num_craft_list_items = self._io.read_u4le()
            self.craft_list = []
            for i in range(self.num_craft_list_items):
                self.craft_list.append(Houseinfo.CraftListType(self._io, self, self._root))

            self.pad1 = self._io.read_u1()
