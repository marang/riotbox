# Pinned DAWproject XML schemas

Unmodified `Project.xsd` and `MetaData.xsd` from
[bitwig/dawproject at ee4dcdde75940f30e14e55401a26955a58b8322b](https://github.com/bitwig/dawproject/tree/ee4dcdde75940f30e14e55401a26955a58b8322b).
The upstream MIT license is included as `LICENSE`.

RIOTBOX-1494 / RBX-373 uses these with `xmllint --nonet` to independently
validate synthetic W-30 and live-master export XML. Tests never fetch schemas
or use source/holdout audio. This checks emitted XML against the external
schema, not import/playback in a DAW.
