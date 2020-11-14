#! /usr/bin/env bash
set -euo pipefail

echo "Fetching deepspeech models"
mkdir -p models
wget --quiet -P models 'https://github.com/mozilla/DeepSpeech/releases/download/v0.9.1/deepspeech-0.9.1-models.pbmm'
wget --quiet -P models 'https://github.com/mozilla/DeepSpeech/releases/download/v0.9.1/deepspeech-0.9.1-models.scorer'

echo "Fetching deepspeech native_client"
mkdir -p deepspeech-lib
wget --quiet -P deepspeech-lib 'https://github.com/mozilla/DeepSpeech/releases/download/v0.9.1/native_client.amd64.cpu.linux.tar.xz'
pushd deepspeech-lib
tar -xaf './native_client*'
popd

echo "Done"

