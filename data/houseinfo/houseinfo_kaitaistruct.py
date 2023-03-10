""" This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild
"""
# pylint: skip-file

import kaitaistruct
from kaitaistruct import KaitaiStruct

if getattr(kaitaistruct, 'API_VERSION', (0, 9)) < (0, 9):
    raise Exception("Incompatible Kaitai Struct Python API: 0.9 or later is required,",
                    f" but you have {kaitaistruct.__version__}")


class Houseinfo(KaitaiStruct):
    """ The houseinfo class for KaitaiStruct
    """

    def __init__(self, _io, _parent=None, _root=None):
        self._io = _io
        self._parent = _parent
        self._root = _root if _root else self
        self._read()

    def _read(self):
        self.header = (self._io.read_bytes(4)).decode("UTF-8")
        self.num_entries = self._io.read_u4le()
        self.house_info = []
        for _ in range(self.num_entries):
            self.house_info.append(Houseinfo.HouseinfoType(self._io, self, self._root))

    class CraftListType(KaitaiStruct):
        """ The CraftList type class for KaitaiStruct
        """

        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.item_craft_index = self._io.read_u4le()
            self.house_level = self._io.read_u4le()

    class NeedHouseKeyType(KaitaiStruct):
        """ The NeedHouseKey type class for KaitaiStruct
        """

        def __init__(self, _io, _parent=None, _root=None):
            self._io = _io
            self._parent = _parent
            self._root = _root if _root else self
            self._read()

        def _read(self):
            self.need_house_key = self._io.read_u2le()

    class HouseinfoType(KaitaiStruct):
        """ The houseinfo type class for KaitaiStruct
        """

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
            for _ in range(2):
                self.unk2.append(self._io.read_u1())

            self.house_group = self._io.read_u4le()
            self.house_floor = self._io.read_u4le()
            self.has_need_house_key = self._io.read_u4le()
            self.unk3 = self._io.read_u4le()
            if self.has_need_house_key == 1:
                self.need_house_key = Houseinfo.NeedHouseKeyType(self._io, self, self._root)

            self.num_craft_list_items = self._io.read_u4le()
            self.craft_list = []
            for _ in range(self.num_craft_list_items):
                self.craft_list.append(Houseinfo.CraftListType(self._io, self, self._root))
