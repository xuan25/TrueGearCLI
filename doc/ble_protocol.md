# True Gear BLE protocol

This document describes the BLE protocol used to communicate with True Gear devices. 

> WARNING: This document is based on the reverse engineering of the `True Gear ME02` and may not be complete or accurate.

The data is sent over BLE in a binary format to control the device. The client need to find the service `SERVICE_UUID_CENTER` and the characteristic `SERVICE_UUID_CENTER_CHARACTERISTICS` of the service to write the data.

```sh
SERVICE_UUID_CENTER = "6e400001-b5a3-f393-e0a9-e50e24dcca9e"
SERVICE_UUID_CENTER_CHARACTERISTICS = "6e400002-b5a3-f393-e0a9-e50e24dcca9e"
```

The basic unit of communication is the EffectObject, which contains multiple TrackObjects. Each TrackObject defines a specific effect to be applied to the device.

```
|=========================================================================|
|                             EffectObject                                |
|=========================================================================|
|   idx   |   0   |   1   |   2   |   3   |   4   |   5   |   6   |   7   |
|-------------------------------------------------------------------------|
|  0x00   | 0x68                                                          |
|-------------------------------------------------------------------------|
|  0x01   | 0x68                                                          |
|-------------------------------------------------------------------------|
|  0x02   | N                                                             |
|-------------------------------------------------------------------------|
|  0x03   | TrackObject 1                                                 |
|  ...    |                                                               |
|  0x12   |                                                               |
|-------------------------------------------------------------------------|
|  0x13   | TrackObject 2                                                 |
|  ...    |                                                               |
|  0x22   |                                                               |
|-------------------------------------------------------------------------|
|  ...    | ...                                                           |
|-------------------------------------------------------------------------|
|  0xN3   | TrackObject N                                                 |
|  ...    |                                                               |
|  0xN2   |                                                               |
|-------------------------------------------------------------------------|
|  0xN3   | 0x16                                                          |
|=========================================================================|

N:
    number of TrackObjects included in this EffectObject
```

```
action_type = Electrical
|=========================================================================|
|                    TrackObject Electical Segment                        |
|=========================================================================|
|   idx   |   0   |   1   |   2   |   3   |   4   |   5   |   6   |   7   |
|-------------------------------------------------------------------------|
|   0x0   | intensity_mode                                                |
|-------------------------------------------------------------------------|
|   0x1   | 0x00                                                          |
|-------------------------------------------------------------------------|
|   0x2   | start_time (big endian)                                       |
|   0x3   |                                                               |
|-------------------------------------------------------------------------|
|   0x4   | end_time (big endian)                                         |
|   0x5   |                                                               |
|-------------------------------------------------------------------------|
|   0x6   | interval                                                      |
|-------------------------------------------------------------------------|
|   0x7   | 0x00                                                          |
|-------------------------------------------------------------------------|
|   0x8   | start_intensity (big endian)                                  |
|   0x9   |                                                               |
|-------------------------------------------------------------------------|
|   0xA   | end_intensity (big endian)                                    |
|   0xB   |                                                               |
|-------------------------------------------------------------------------|
|   0xC   | group_left                                                    |
|-------------------------------------------------------------------------|
|   0xD   | 0x00                                                          |
|-------------------------------------------------------------------------|
|   0xE   | group_right                                                   |
|-------------------------------------------------------------------------|
|   0xF   | 0x00                                                          |
|=========================================================================|

intensity_mode values:
    0x10 = (once = true)
    0x11 = Const
    0x12 = Fade / FadeInAndOut

group values:
    0xF0 = on
    0x00 = off

Note: under Once mode
    time_end, interval and intensity_end are set to 0
```

```
action_type = Shake
|=========================================================================|
|                       TrackObject Shake Segment                         |
|=========================================================================|
|   idx   |   0   |   1   |   2   |   3   |   4   |   5   |   6   |   7   |
|-------------------------------------------------------------------------|
|   0x0   | intensity_mode                                                |
|-------------------------------------------------------------------------|
|   0x1   | register_id/0x00                                              |
|-------------------------------------------------------------------------|
|   0x2   | start_time (big endian)                                       |
|   0x3   |                                                               |
|-------------------------------------------------------------------------|
|   0x4   | end_time (big endian)                                         |
|   0x5   |                                                               |
|-------------------------------------------------------------------------|
|   0x6   | start_intensity                                               |
|-------------------------------------------------------------------------|
|   0x7   | end_intensity                                                 |
|-------------------------------------------------------------------------|
|   0x8   | dot_group_front_left (big endian)                             |
|   0x9   |                                                               |
|-------------------------------------------------------------------------|
|   0xA   | dot_group_back_left (big endian)                              |
|   0xB   |                                                               |
|-------------------------------------------------------------------------|
|   0xC   | dot_group_back_right (big endian)                             |
|   0xD   |                                                               |
|-------------------------------------------------------------------------|
|   0xE   | dot_group_front_right (big endian)                            |
|   0xF   |                                                               |
|=========================================================================|

intensity_mode values:
    0x01 = Const + not keep
    0x02 = Fade / FadeInAndOut + not keep
    0x03 = Const + keep
    0x04 = Fade / FadeInAndOut + keep

register_id:
    if the effect is not with keep mode, use 0 as id
    if the effect is with keep mode
    Note: the usage of register_id on the device-side is not clear yet

dot_group values:
    each dot group is a short (2 bytes) representing 16 dots maximum
    each bit in the short represents whether a dot is activated (1) or not (0)
    the most significant bit (1<<15) represents the first dot in the group (top-left)
    the dots are enumerated from top-left to bottom-right, y first enumeration
    for front dots, we view the device from the front side (mirrored from actual positions)
    for back dots, we view the device from the back side (normal positions)

```

## Exmples

```
Connected
6868010101000000641414ffc0ffc0ffc0ffc016
686801100000000000000000030000f000f00016
6868010101000000641414ffc0ffc0ffc0ffc016
686801100000000000000000030000f000f00016

Test
6868010200000001f40046ffc0ffc0ffc0ffc016
686801111602bf03e8010000320032f000f00016

Electrical Stimulation Intensity; High-Intensity Testing; 10%
6868011100000001f41400000a000af000f00016

Electrical Stimulation Intensity; Low-Intensity Testing; 10%
6868011000000000000000000a0000f000f00016

Electrical Stimulation Intensity; High-Intensity Testing; 100%
6868011100000001f4140000630063f000f00016

Electrical Stimulation Intensity; Low-Intensity Testing; 100%
686801100000000000000000630000f000f00016

Electrical Stimulation Intensity; High-Intensity Testing; 150%
6868011100000001f4140000960096f000f00016

Electrical Stimulation Intensity; Low-Intensity Testing; 150%
686801100000000000000000960000f000f00016
```
