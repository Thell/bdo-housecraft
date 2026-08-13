# This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild
# type: ignore

import kaitaistruct
from kaitaistruct import KaitaiStruct, KaitaiStream, BytesIO


if getattr(kaitaistruct, 'API_VERSION', (0, 9)) < (0, 11):
    raise Exception("Incompatible Kaitai Struct Python API: 0.11 or later is required, but you have %s" % (kaitaistruct.__version__))

class Houseinfo(KaitaiStruct):
    def __init__(self, _io, _parent=None, _root=None):
        super(Houseinfo, self).__init__(_io)
        self._parent = _parent
        self._root = _root or self
        self._read()

    def _read(self):
        self.header = (self._io.read_bytes(4)).decode(u"UTF-8")
        self.num_entries = self._io.read_u4le()
        self.house_info = []
        for i in range(self.num_entries):
            self.house_info.append(Houseinfo.HouseinfoType(self._io, self, self._root))



    def _fetch_instances(self):
        pass
        for i in range(len(self.house_info)):
            pass
            self.house_info[i]._fetch_instances()


    class CraftListType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            super(Houseinfo.CraftListType, self).__init__(_io)
            self._parent = _parent
            self._root = _root
            self._read()

        def _read(self):
            self.item_craft_index = self._io.read_u4le()
            self.house_level = self._io.read_u4le()


        def _fetch_instances(self):
            pass


    class HouseinfoType(KaitaiStruct):
        def __init__(self, _io, _parent=None, _root=None):
            super(Houseinfo.HouseinfoType, self).__init__(_io)
            self._parent = _parent
            self._root = _root
            self._read()

        def _read(self):
            self.need_explore_point = self._io.read_u2le()
            self.affiliated_town = self._io.read_u2le()
            self.parent_node = self._io.read_u2le()
            self.unk1 = self._io.read_u2le()
            self.character_key = self._io.read_u2le()
            self.unk2 = self._io.read_u2le()
            self.house_group = self._io.read_u4le()
            self.house_floor = self._io.read_u4le()
            self.len_need_house_key = self._io.read_u8le()
            self.need_house_key = []
            for i in range(self.len_need_house_key):
                self.need_house_key.append(self._io.read_u2le())

            self.num_craft_list_items = self._io.read_u4le()
            self.craft_list = []
            for i in range(self.num_craft_list_items):
                self.craft_list.append(Houseinfo.CraftListType(self._io, self, self._root))

            self.pad1 = self._io.read_u1()


        def _fetch_instances(self):
            pass
            for i in range(len(self.need_house_key)):
                pass

            for i in range(len(self.craft_list)):
                pass
                self.craft_list[i]._fetch_instances()




