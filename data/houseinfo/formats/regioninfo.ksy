# -----------------------------
# This file is auto-generated by:
# bdo-auto-ksy version: 2.3
# -----------------------------

meta:
  id: regioninfo
  file-extension: bss
  endian: le

seq:
  - id: pabr
    type: str
    encoding: utf-8
    size: 4

  - id: num_regioninfo
    type: u4

  - id: regioninfo_table
    type: regioninfo_type
    repeat: expr
    repeat-expr: num_regioninfo

  - id: string_table
    type: string_table_type

types:
  regioninfo_type:
    seq:
      - id: id
        type: u2

      - id: color_key
        type: color_type

      - id: region_type
        type: u1
        enum: region_type

      - id: village_siege_type
        type: u4
        enum: village_siege_type

      - id: unknown_enum_type_1_nws
        type: u1

      - id: is_safe_zone
        type: u1

      - id: is_arena_zone
        type: u1

      - id: is_sea
        type: u1

      - id: is_desert
        type: u1

      - id: is_not_dig
        type: u1

      - id: is_ocean
        type: u1

      - id: is_prison
        type: u1

      - id: is_king_war_zone_only_unsure
        type: u1

      - id: is_lord_war_zone_only_unsure
        type: u1

      - id: is_village_war_area
        type: u1

      - id: is_king_or_lord_war_zone
        type: u1

      - id: is_main_town
        type: u1

      - id: is_minor_town
        type: u1

      - id: is_main_or_minor_town
        type: u1

      - id: accessible_area
        type: u1

      - id: is_node_access
        type: u1

      - id: village_tax_level
        type: u1

      - id: unknown_1_nws
        type: u2
        doc: poss village_tax_level_sum u4

      - id: unknown_2
        type: u4
        doc: all are 0x57E4 (22500)

      - id: unknown_3
        type: u1
        doc: all are 0x00

      - id: unknown_bool_indicator_1
        type: u1
        doc: there are 110 '0' indicators

      - id: villain_respawn_explore_key
        type: u4

      - id: villain_respawn_postion
        type: position_type

      - id: is_ancient_dungeon
        type: u1

      - id: is_escape
        type: u1

      - id: is_special_zone
        type: u1

      - id: vehicle_dead_zone
        type: u1

      - id: region_skill_1
        type: region_skill_type
        
      - id: region_skill_2
        type: region_skill_type

      - id: region_skill_3
        type: region_skill_type
        
      - id: region_night_skill
        type: region_skill_type

      - id: territory_key
        type: u2

      - id: index_area_name
        type: u4

      - id: index_return_position_name
        type: u4

      - id: affiliated_town_region_key
        type: u2

      - id: trade_origin_region_key
        type: u2

      - id: region_group_key
        type: u2

      - id: immediate_respawn
        type: u1
        # 106 bytes

      - id: waypoint_key
        type: u4

      - id: explore_key
        type: u4

      - id: region_attribute_type
        type: u4
        enum: region_attribute_type

      - id: center_position
        type: position_type

      - id: return_position
        type: position_type

      - id: valid_return_position_list_len
        type: u4
        # 146

      - id: valid_return_position_list
        type: position_type
        repeat: expr
        repeat-expr: valid_return_position_list_len
        # 146 + valid_return_position_list_len * 12

      - id: exploration_point
        type: u2

      - id: weather_data
        type: weather_data_type

      - id: warehouse_character_key
        type: u2
        # 146 + valid_return_position_list_len * 12 + 38

      - id: warehouse_dialog_index
        type: u2

      - id: npc_worker_character_key
        type: u2

      - id: npc_worker_dialog_index
        type: u2

      - id: stable_character_key
        type: u2

      - id: sable_dialog_index
        type: u2

      - id: wharf_character_key
        type: u2

      - id: wharf_dialog_index
        type: u2

      - id: item_market_character_key
        type: u2

      - id: item_market_dialog_index
        type: u2

      - id: delivery_character_key
        type: u2

      - id: delivery_dialog_index
        type: u2

      - id: is_free_revival_area
        type: u1
        # 20

      - id: pc_deliver_region_key_list_len
        type: u4
        # 21

      - id: pc_deliver_region_key_list
        type: field_list_type
        repeat: expr
        repeat-expr: pc_deliver_region_key_list_len

      - id: respawn_position_list_len
        type: u4

      - id: respawn_position_list
        type: position_type
        repeat: expr
        repeat-expr: respawn_position_list_len

      # 176B to end of record
      - id: unknown_5
        type: u1
        doc: all 0x00

      - id: unknown_6
        type: u2
        doc: all 0xFFFF

      - id: unknown_7_nws
        type: u2
        doc: unknown increment scalar

      - id: unknown_8_nws
        type: u4

      - id: unknown_9_nws
        type: u4

      - id: unknown_10_nws
        type: u4

      - id: unknown_11_nws
        type: u4

      - id: unknown_12_nws
        type: u2

      - id: unknown_13_nws
        type: u1

      - id: world_boss_reward
        type: u1

      - id: escape_point
        type: position_type

      - id: revive_point
        type: position_type

      - id: alignment_padding
        type: u8
        doc: buffer size is managed after the revive_point

      - id: unknown_14_nws
        type: u4
        doc: last block seems all node war and siege related

      - id: unknown_15_nws
        type: u4

      - id: unknown_16
        type: u4
        doc: all 0x00

      - id: is_pit_of_the_undying
        type: u1

      - id: premium_character_possible_region
        type: u1

      - id: unknown_17nodewars_siege_related
        type: u4

      - id: unknown_18
        type: u2
        doc: all 0xFFFF

      - id: unknown_19_nws
        type: u4

      - id: unknown_20_nws
        type: u4

      - id: unknown_21nodewars_siege_related
        type: u4

      - id: unknown_22nodewars_siege_related
        type: u4

      - id: unknown_23nodewars_siege_related
        type: u4

      - id: unknown_24_nws
        type: u4

      - id: unknown_25_nws
        type: u4

      - id: unknown_26_nws
        type: u4

      - id: uknown_arena_something
        type: u1
        doc: perhaps something with seige
        
      - id: unknown_27
        size: 12
        doc: actually 8 + 4

      - id: unknown_28_siege_related
        size: 12
        doc: actually 8 + 4
        
      - id: unknown_29_nws
        type: u2
        
      - id: is_siege_challenge
        type: u1

      - id: unknown_30_siege_related
        type: u8
        
      - id: unknown_31_siege_related
        type: u8
        
      - id: unknown_32_nws
        type: u8
        
      - id: unknown_33_nws
        type: u8

    instances:
      area_name:
        value: _root.string_table.strings[index_area_name].string_utf16

  color_type:
    seq:
      - id: b
        type: u1
      - id: g
        type: u1
      - id: r
        type: u1
      - id: a
        type: u1

  field_list_type:
    seq:
      - id: field
        type: u2

  position_type:
    seq:
      - id: x
        type: f4
      - id: y
        type: f4
      - id: z
        type: f4

  region_skill_type:
    seq:
      - id: is_integrity
        type: u2
      - id: skill_key
        type: u2
      - id: apply_rate
        type: s4

  weather_data_type:
    seq:
      - id: is_initialize_weather
        type: u1
      - id: unknown_3
        type: u2
      - id: unknown_4
        type: u1
      - id: temperature
        type: f4
      - id: humility
        type: f4
      - id: oil_rate
        type: f4
      - id: water_rate
        type: f4
      - id: rain_amount
        type: f4
      - id: rain_tick_ready
        type: f4
      - id: rain_tick_keep
        type: f4
      - id: select_weather_time
        type: s4


  string_block_type:
    seq:
      - id: is_utf16_string
        type: u1

      - id: string_length
        type: u4

      - id: string_utf8
        type: str
        encoding: utf-8
        size: string_length
        if: "is_utf16_string == 0"

      - id: string_utf16
        type: str
        encoding: utf-16le
        size: string_length
        if: "is_utf16_string == 1"

  string_table_type:
    seq:
      - id: num_strings
        type: u4

      - id: strings
        type: string_block_type
        repeat: expr
        repeat-expr: num_strings

enums:
  region_type:
    0: minor_town
    1: main_town
    2: hunting
    3: king_war
    4: lord_war
    5: castle
    6: arena
    7: count
    
  village_siege_type:
    0: sunday
    1: monday
    2: tuesday
    3: wednesday
    4: thursday
    5: friday
    6: saturday
    7: count
    
  region_attribute_type:
    0: normal_land
    1: beach
    2: desert
    3: count
    