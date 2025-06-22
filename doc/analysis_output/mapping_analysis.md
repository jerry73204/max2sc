# Max MSP to SuperCollider Object Mapping Analysis

## Summary

- Total patches analyzed: 237
- Unique object types: 419

## Most Used Objects

1. `comment`: 2594 instances
2. `message`: 2509 instances
3. `outlet`: 2261 instances
4. `inlet`: 2033 instances
5. `live.meter~`: 1071 instances
6. `bpatcher`: 1006 instances
7. `adstatus`: 708 instances
8. `thru`: 594 instances
9. `p`: 532 instances
10. `int`: 527 instances
11. `live.text`: 452 instances
12. `loadbang`: 354 instances
13. `number~`: 322 instances
14. `prepend`: 318 instances
15. `live.dial`: 260 instances
16. `live.line`: 251 instances
17. `spat5.osc.route`: 211 instances
18. `t`: 207 instances
19. `umenu`: 187 instances
20. `route`: 183 instances

## Spatial Audio Objects

### SPAT5 Objects
- `spat5.abs2rel`: 1 instances
- `spat5.adc128~`: 1 instances
- `spat5.adm.extractxml`: 1 instances
- `spat5.adm.ltc.encode~`: 2 instances
- `spat5.adm.ltc.slave`: 1 instances
- `spat5.adm.mute`: 4 instances
- `spat5.adm.play~`: 3 instances
  - Example: `spat5.adm.play~ @outlets 24`
- `spat5.adm.record`: 2 instances
  - Example: `spat5.adm.record @inlets 64`
- `spat5.adm.record~`: 2 instances
  - Example: `spat5.adm.record~ @inlets 64`
- `spat5.adm.renderer~`: 3 instances
  - Example: `spat5.adm.renderer~ @inlets 128 @outlets 128 @mc 1`
- `spat5.admix.resources`: 6 instances
- `spat5.align~`: 6 instances
  - Example: `spat5.align~ @speakers 26 @mc 1`
- `spat5.allpass~`: 3 instances
- `spat5.around`: 1 instances
- `spat5.binaural~`: 1 instances
  - Example: `spat5.binaural~ @sources 2 @mc 1`
- `spat5.cascade~`: 7 instances
  - Example: `spat5.cascade~ @channels 24 @mc 1`
- `spat5.checkupdates`: 1 instances
  - Example: `spat5.checkupdates #1`
- `spat5.colormap`: 2 instances
- `spat5.comb~`: 1 instances
  - Example: `spat5.comb~ @channels 4`
- `spat5.compressor`: 1 instances
- `spat5.compressor~`: 1 instances
  - Example: `spat5.compressor~ @mc 1 @channels 2`
- `spat5.constraint`: 1 instances
- `spat5.converter`: 4 instances
  - Example: `spat5.converter @initwith "/format xyz"`
- `spat5.cpu`: 1 instances
  - Example: `spat5.cpu @initwith "/rate 1500"`
- `spat5.dac64~`: 1 instances
- `spat5.decoder~`: 5 instances
  - Example: `spat5.decoder~ @inputs 4 @outputs 3 @initwith "/decoding/type uhj2bformat"`
- `spat5.deferloadbang`: 5 instances
- `spat5.deferloadmess`: 23 instances
  - Example: `spat5.deferloadmess 1`
- `spat5.delay~`: 5 instances
  - Example: `spat5.delay~ @channels 128 @initwith "/interpolation/mode nearest" @mc 1`
- `spat5.delgen`: 1 instances
- `spat5.diagmatrix~`: 26 instances
  - Example: `spat5.diagmatrix~ @channels 128 @mc 1`
- `spat5.dsp.management`: 4 instances
- `spat5.dspstate~`: 3 instances
- `spat5.ebur128~`: 2 instances
  - Example: `spat5.ebur128~ @mc 1 @channels #1`
- `spat5.equalizer`: 9 instances
- `spat5.file.infos`: 14 instances
- `spat5.filterdesign`: 1 instances
  - Example: `spat5.filterdesign @initwith "/topology butterworth, /order 10, /frequency 40, /response highpass"`
- `spat5.flux.reverb.presets`: 2 instances
- `spat5.frequencyresponse.embedded`: 4 instances
- `spat5.gammatone`: 1 instances
  - Example: `spat5.gammatone @initwith "/order 1"`
- `spat5.gate~`: 10 instances
  - Example: `spat5.gate~ @outputs 24 @mc 1`
- `spat5.granulator~`: 1 instances
  - Example: `spat5.granulator~ @outputs 16 @mc 1`
- `spat5.grids`: 1 instances
- `spat5.hlshelf`: 1 instances
- `spat5.hlshelf.embedded`: 1 instances
- `spat5.hlshelf~`: 1 instances
  - Example: `spat5.hlshelf~ @mc 1 @channels 25`
- `spat5.hoa.beam~`: 1 instances
  - Example: `spat5.hoa.beam~ @order 4 @dimension 3 @mc 1 @initwith "/norm N3D, /beam/*/pattern omni"`
- `spat5.hoa.binaural~`: 1 instances
  - Example: `spat5.hoa.binaural~ @dimension 3 @order 3 @mc 1 @initwith "/norm SN3D"`
- `spat5.hoa.blur~`: 3 instances
  - Example: `spat5.hoa.blur~ @mc 1 @dimension 3 @order 4`
- `spat5.hoa.converter~`: 13 instances
  - Example: `spat5.hoa.converter~ @mc 1 @dimension 3 @order 6`
- `spat5.hoa.decoder~`: 9 instances
  - Example: `spat5.hoa.decoder~ @outputs 64 @order 7 @dimension 3 @mc 1`
- `spat5.hoa.directivity`: 1 instances
  - Example: `spat5.hoa.directivity @initwith "/order 3"`
- `spat5.hoa.display.embedded`: 3 instances
- `spat5.hoa.em32~`: 2 instances
  - Example: `spat5.hoa.em32~ @mc 1 @initwith "/norm N3D"`
- `spat5.hoa.encoder~`: 16 instances
  - Example: `spat5.hoa.encoder~ @order 4 @dimension 3 @mc 1`
- `spat5.hoa.focus`: 1 instances
- `spat5.hoa.focus.embedded`: 1 instances
- `spat5.hoa.focus~`: 2 instances
  - Example: `spat5.hoa.focus~ @mc 1 @dimension 3 @order 4`
- `spat5.hoa.mirror~`: 2 instances
  - Example: `spat5.hoa.mirror~ @mc 1 @dimension 3 @order 4`
- `spat5.hoa.plot`: 3 instances
  - Example: `spat5.hoa.plot @initwith "/order 3, /norm N3D, /display/zoom 0.5"`
- `spat5.hoa.reduce~`: 2 instances
  - Example: `spat5.hoa.reduce~ @order 7 @dimension 3D @mc 1`
- `spat5.hoa.rotate~`: 21 instances
  - Example: `spat5.hoa.rotate~ @mc 1 @dimension 3 @order 4`
- `spat5.hoa.scope~`: 3 instances
  - Example: `spat5.hoa.scope~ @mc 1 @dimension 3 @order 4`
- `spat5.hoa.slaconv~`: 1 instances
  - Example: `spat5.hoa.slaconv~ @order 3 @speakers 20 @mc 1`
- `spat5.hoa.sorting~`: 14 instances
  - Example: `spat5.hoa.sorting~ @mc 1 @dimension 3 @order 6`
- `spat5.hoa.warp~`: 1 instances
  - Example: `spat5.hoa.warp~ @mc 1 @dimension 3 @order 4`
- `spat5.hostinfos`: 3 instances
- `spat5.iko.decoder~`: 1 instances
- `spat5.io.mapping.1-256`: 2 instances
  - Example: `spat5.io.mapping.1-256 input`
- `spat5.io.mapping.257-512`: 2 instances
  - Example: `spat5.io.mapping.257-512 input`
- `spat5.io.mappings`: 3 instances
- `spat5.jitter`: 1 instances
- `spat5.ltc.decode~`: 2 instances
- `spat5.ltc.display`: 3 instances
- `spat5.ltc.easydecode~`: 1 instances
  - Example: `spat5.ltc.easydecode~ @initwith "/rate 70"`
- `spat5.ltc.encode~`: 1 instances
- `spat5.ltc.trigger~`: 1 instances
- `spat5.mc.adc128~`: 2 instances
- `spat5.mc.adc64~`: 1 instances
- `spat5.mc.dac128~`: 4 instances
- `spat5.mc.dac192~`: 1 instances
- `spat5.mc.dac256~`: 1 instances
- `spat5.mc.dac512~`: 1 instances
- `spat5.mc.dac64~`: 4 instances
- `spat5.mc.test.signal~`: 2 instances
- `spat5.mcsfplayer128~`: 1 instances
  - Example: `spat5.mcsfplayer128~ 128`
- `spat5.mcsfplayer64~`: 1 instances
  - Example: `spat5.mcsfplayer64~ 64`
- `spat5.mcsfplayer~`: 1 instances
  - Example: `spat5.mcsfplayer~ 64`
- `spat5.meter~`: 5 instances
  - Example: `spat5.meter~ @channels 128 @mc 1`
- `spat5.mirror`: 1 instances
- `spat5.normalize`: 4 instances
- `spat5.ntof`: 8 instances
- `spat5.oper`: 11 instances
  - Example: `spat5.oper @initwith " /source/number 1, /internals 8"`
- `spat5.oper_`: 1 instances
  - Example: `spat5.oper_ @initwith " /source/number 1, /internals 8"`
- `spat5.osc.append`: 19 instances
  - Example: `spat5.osc.append /z`
- `spat5.osc.collect`: 2 instances
- `spat5.osc.display`: 3 instances
- `spat5.osc.flip`: 2 instances
- `spat5.osc.ignore`: 22 instances
  - Example: `spat5.osc.ignore /track/*/editable`
- `spat5.osc.iter`: 23 instances
- `spat5.osc.prepend`: 140 instances
  - Example: `spat5.osc.prepend /values`
- `spat5.osc.print`: 5 instances
- `spat5.osc.replace`: 46 instances
- `spat5.osc.route`: 211 instances
  - Example: `spat5.osc.route /track/11 /track/12 /track/13 /track/14 /track/15 /track/16 /track/17 /track/18 /track/19 /track/20`
- `spat5.osc.routepass`: 95 instances
  - Example: `spat5.osc.routepass /speakers /speaker`
- `spat5.osc.slashify`: 3 instances
- `spat5.osc.speedlim`: 3 instances
  - Example: `spat5.osc.speedlim @rate 0`
- `spat5.osc.split`: 27 instances
  - Example: `spat5.osc.split @initwith "/stride 1"`
- `spat5.osc.todict`: 3 instances
- `spat5.osc.trim`: 1 instances
  - Example: `spat5.osc.trim -1`
- `spat5.osc.udpreceive`: 1 instances
  - Example: `spat5.osc.udpreceive @port 6161`
- `spat5.osc.udpsend`: 1 instances
  - Example: `spat5.osc.udpsend @ip 192.168.68.65 @port 6969`
- `spat5.osc.unslashify`: 8 instances
- `spat5.osc.var`: 2 instances
  - Example: `spat5.osc.var @embed 1`
- `spat5.osc.view`: 61 instances
- `spat5.pan`: 3 instances
  - Example: `spat5.pan @inputs 1 @outputs 14 @initwith "/panning/type vbap3d, /phantom/nadir 1"`
- `spat5.panoramix`: 7 instances
  - Example: `spat5.panoramix @inlets 21 @outlets 82`
- `spat5.panoramix.resources`: 3 instances
- `spat5.panoramix.reverb.presets`: 1 instances
- `spat5.panoramix.speaker.directions`: 1 instances
- `spat5.panoramix.speaker.layout`: 3 instances
- `spat5.panoramix2tosca`: 3 instances
- `spat5.panoramix~`: 6 instances
  - Example: `spat5.panoramix~ @inlets 21 @outlets 82 @mc 1`
- `spat5.pan~`: 20 instances
  - Example: `spat5.pan~ @inputs 2 @outputs 6`
- `spat5.pattr`: 1 instances
- `spat5.ping`: 1 instances
- `spat5.pink~`: 4 instances
  - Example: `spat5.pink~ @channels 64 @mc 1`
- `spat5.plot`: 4 instances
- `spat5.positions.maxhelp`: 1 instances
- `spat5.presets.management`: 2 instances
- `spat5.quat.fromeuler`: 1 instances
- `spat5.quat.toeuler`: 3 instances
  - Example: `spat5.quat.toeuler @initwith "/mode zyx"`
- `spat5.rake~`: 1 instances
  - Example: `spat5.rake~ @outputs 16 @mc 1`
- `spat5.rms~`: 27 instances
  - Example: `spat5.rms~ @channels 128 @mc 1`
- `spat5.rotate`: 2 instances
- `spat5.routing`: 9 instances
  - Example: `spat5.routing @inputs 128 @outputs 128`
- `spat5.routing~`: 9 instances
  - Example: `spat5.routing~ @inputs 128 @outputs 128 @mc 1`
- `spat5.scale`: 10 instances
  - Example: `spat5.scale @initwith "/scaling/dist 10"`
- `spat5.sfplay~`: 6 instances
  - Example: `spat5.sfplay~ @mc 1 @channels 2 @initwith "/loop 1"`
- `spat5.sfrecord~`: 7 instances
  - Example: `spat5.sfrecord~ @channels 128 @mc 1`
- `spat5.sig~`: 6 instances
  - Example: `spat5.sig~ @channels 64 @mc 1`
- `spat5.simone`: 1 instances
  - Example: `spat5.simone @rows 25 @cols 25 @initwith "/window/floating 1, /window/size 800 800"`
- `spat5.simone.generator`: 1 instances
  - Example: `spat5.simone.generator @rows 25 @cols 25 @initwith "/type A"`
- `spat5.smk~`: 1 instances
  - Example: `spat5.smk~ @inlets 32 @mc 1`
- `spat5.snapshot.management`: 2 instances
- `spat5.snapshot~`: 13 instances
  - Example: `spat5.snapshot~ @channels 4 @initwith "/rate 50"`
- `spat5.sofa.loader`: 3 instances
- `spat5.spat~`: 11 instances
  - Example: `spat5.spat~ @inputs 2 @outputs 4 @rooms 1 @initwith "/panning/type vbap2d"`
- `spat5.speaker.config`: 3 instances
- `spat5.speaker.layout`: 6 instances
  - Example: `spat5.speaker.layout @initwith "/type 'ircam studio 4'"`
- `spat5.sprintf`: 5 instances
- `spat5.tapout~`: 1 instances
  - Example: `spat5.tapout~ @channels 2`
- `spat5.test.dac128~`: 3 instances
- `spat5.test.dac64~`: 6 instances
- `spat5.test.signal~`: 1 instances
- `spat5.thru128~`: 6 instances
- `spat5.thru32~`: 4 instances
- `spat5.thru64~`: 9 instances
- `spat5.thru96~`: 2 instances
- `spat5.trajectories`: 12 instances
  - Example: `spat5.trajectories @initwith "/type squareknot, /radius 0.25"`
- `spat5.transform`: 6 instances
- `spat5.translate`: 13 instances
  - Example: `spat5.translate @initwith "/offset/z -1.6"`
- `spat5.version`: 2 instances
- `spat5.viewer`: 95 instances
- `spat5.viewer.embedded`: 1 instances
- `spat5.viewer.options`: 1 instances
- `spat5.virtualspeakers~`: 7 instances
  - Example: `spat5.virtualspeakers~ @speakers 6 @mc 1`
- `spat5.window.management`: 1 instances

## Multichannel Objects

- `mc.*~`: 1 instances
- `mc.+~`: 1 instances
- `mc.adc~`: 7 instances
- `mc.channelcount~`: 7 instances
- `mc.combine~`: 3 instances
- `mc.dac~`: 20 instances
- `mc.gate~`: 1 instances
- `mc.live.gain~`: 56 instances
- `mc.pack~`: 22 instances
- `mc.poly~`: 1 instances
- `mc.separate~`: 1 instances
- `mc.sig~`: 2 instances
- `mc.snapshot~`: 1 instances
- `mc.unpack~`: 13 instances

## Audio I/O Objects

- `adc~`: 8 instances
- `dac~`: 26 instances
- `ezdac~`: 3 instances
- `sfplay~`: 3 instances

## Routing Objects

- `gate~`: 4 instances
- `matrix~`: 2 instances
- `route`: 183 instances
- `selector~`: 3 instances