<h1 align="center">DJ-4LED</h1>
<p align="center">
  Visualiseur LED Audio Temps Réel
</p>

<p align="center">
  <a href="#introduction"><strong>Introduction</strong></a> ·
  <a href="#fonctionnalités"><strong>Fonctionnalités</strong></a> ·
  <a href="#démarrage-rapide"><strong>Démarrage Rapide</strong></a> ·
  <a href="#tech-stack"><strong>Tech Stack</strong></a> ·
  <a href="#configuration"><strong>Configuration</strong></a> ·
  <a href="#effets-visuels"><strong>Effets Visuels</strong></a>
</p>
<br/>

## Introduction

DJ-4LED est un système de visualisation LED haute performance qui synchronise un panneau LED avec de l'audio en temps réel. Conçu pour les événements, concerts et installations artistiques, il utilise le protocole ArtNet pour contrôler plusieurs contrôleurs simultanément.

Le cœur du système repose sur une analyse FFT haute performance et un moteur de rendu optimisé pour délivrer des effets visuels synchronisés à 60 FPS. Que ce soit pour des barres spectrales réactives, des ondes circulaires ou autre, DJ-4LED transforme l'audio en spectacle visuel immersif.

## Fonctionnalités

**Capture audio temps réel** - Analyse FFT haute performance avec latence <10ms

**Effets visuels multiples** - Barres spectrales, ondes circulaires, système de particules

**Performance optimisée** - Jusqu'à 60 FPS avec multithreading

**Interface UDP** - Contrôle à distance via interface udp en temps réel

**Configuration flexible** - Paramètres audio, LED et effets personnalisables

**Protocole ArtNet** - Compatible avec l'éclairage professionnel standard

## Démarrage Rapide

Pour configurer DJ-4LED localement, vous devez cloner le repository et vous assurer d'avoir les prérequis suivants :

**Prérequis :**

- Rust 1.70+ ([Installation](https://rustup.rs/))
- macOS/Linux (testé sur macOS)
- Node.js (optionnel, pour l'interface web)

**Installation :**

```bash
git clone https://github.com/clementpnn/dj-4led.git
cd dj-4led
```

**Démarrage :**

```bash
cd apps/backend
cargo build --release
./target/release/led-visualizer --production
```

## Tech Stack

DJ-4LED est construit sur les technologies suivantes :

**Backend :**

- [Rust](https://www.rust-lang.org/) - Langage de programmation principal
- [Cpal](https://crates.io/crates/cpal) - Capture audio multiplateforme
- [FFT](https://crates.io/crates/rustfft) - Analyse spectrale temps réel

**Frontend :**

- [TypeScript](https://www.typescriptlang.org/) - Langage de programmation
- [Vue.js](https://vuejs.org/) - Framework JavaScript
- [Tauri](https://tauri.app/) - Framework d'application desktop

**Protocoles & Hardware :**

- [ArtNet](https://art-net.org.uk/) - Protocole d'éclairage professionnel
- [UDP](https://en.wikipedia.org/wiki/User_Datagram_Protocol) - Transport réseau
- Contrôleurs LED compatibles ArtNet
