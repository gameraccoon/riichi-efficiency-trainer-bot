# Riichi Efficiency Trainer Bot for Telegram
[![Builds and Tests](https://github.com/gameraccoon/riichi-trainer/actions/workflows/rust.yml/badge.svg)](https://github.com/gameraccoon/riichi-trainer/actions/workflows/rust.yml)

A small app to train different skills related to playing riichi mahjong

Shanten calculations are taken from https://github.com/Euophrys/Riichi-Trainer, however, there are a few noticeable differences:
- Ukeire2 is calculated instead of Ukeire1 (2 moves ahead instead of 1 move)
- Final scoring is more distributed (doing random moves is more punishing for the final score)
