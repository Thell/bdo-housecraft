meta:
  id: mansionlandinfo
  file-extension: bss
  endian: le
  
seq:
  - id: header
    type: str
    encoding: UTF-8
    size: 4

  - id: num_offset_table
    type: u4

  - id: mansionlandinfo_table
    type: mansionlandinfo_type
    repeat: expr
    repeat-expr: num_offset_table

types:
  mansionlandinfo_type:
    seq:
      - id: land_character_key
        type: u2
        
      - id: building_character_key
        type: u2
        
      - id: need_point
        type: u2
        
      - id: region
        type: u2
        
      - id: sector_start_position
        type: s4
        repeat: expr
        repeat-expr: 3

      - id: sector_end_position
        type: s4
        repeat: expr
        repeat-expr: 3

      - id: unknown_sector_position_1
        type: s4
        repeat: expr
        repeat-expr: 3

      - id: unknown_sector_position_2
        type: s4
        repeat: expr
        repeat-expr: 3
  
      - id: condition
        type: u2
        repeat: expr
        repeat-expr: 2

      - id: day_pay
        type: u8

      - id: unknown_1
        type: f4
  
      - id: unknown_2
        type: f4
  
      - id: unknown_3
        type: f4
  
      - id: mansion_world_map_icon_type
        type: u1
  
      - id: guild_mansion_teleport_position
        type: position_type

  position_type:
    seq:
      - id: x
        type: f4
      - id: y
        type: f4
      - id: z
        type: f4
