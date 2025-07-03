# 🎵 DJ-4LED - Visualiseur LED Audio Temps Réel

![DJ-4LED](https://img.shields.io/badge/DJ--4LED-v1.0-brightgreen)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
![License](https://img.shields.io/badge/License-MIT-blue)

DJ-4LED est un système de visualisation LED haute performance qui synchronise des panneaux LED avec de l'audio en temps réel. Conçu pour les événements, concerts et installations artistiques, il utilise le protocole ArtNet pour contrôler jusqu'à 4 contrôleurs LED simultanément.

## ✨ Fonctionnalités

- 🎧 **Capture audio temps réel** - Analyse FFT haute performance
- 🌈 **Effets visuels multiples** - Barres spectrales, ondes circulaires, système de particules
- 🚀 **Performance optimisée** - Jusqu'à 60 FPS avec multithreading
- 🌐 **Interface WebSocket** - Contrôle à distance via interface web
- 🔧 **Configuration flexible** - Paramètres audio, LED et effets personnalisables
- 📡 **Protocole ArtNet** - Compatible avec l'éclairage professionnel
- 🎮 **Modes multiples** - Test, développement et production

## 🏗️ Architecture

```
DJ-4LED/
├── apps/
│   └── backend/           # Backend Rust (capture audio + rendu LED)
│       ├── src/
│       │   ├── main.rs    # Point d'entrée principal
│       │   ├── audio.rs   # Capture et analyse audio
│       │   ├── effects.rs # Effets visuels
│       │   ├── led.rs     # Contrôleur LED ArtNet
│       │   └── config.rs  # Configuration
│       └── config.toml    # Configuration système
├── start_backend.sh       # Script pour démarrer le backend seul
└── start_vivid.sh        # Script complet avec interface web
```

## 🚀 Démarrage Rapide

### Prérequis

- **Rust** 1.70+ ([Installation](https://rustup.rs/))
- **macOS/Linux** (testé sur macOS)
- **Node.js** (optionnel, pour l'interface web)

### Installation

1. **Cloner le projet**
   ```bash
   git clone <repository-url>
   cd dj-4led
   ```

2. **Rendre les scripts exécutables**
   ```bash
   chmod +x start_backend.sh start_vivid.sh
   ```

### Démarrage

#### Option 1 : Backend uniquement
```bash
./start_backend.sh
```

#### Option 2 : Backend + Interface Web
```bash
./start_vivid.sh
```

#### Option 3 : Démarrage manuel
```bash
cd apps/backend
cargo build --release
./target/release/led-visualizer --production
```

## 🎛️ Modes de Fonctionnement

### Mode Test (Automatique)
- Activé quand aucun contrôleur LED n'est détecté
- Audio simulé pour les tests
- Idéal pour le développement

### Mode Production
- Détection automatique des contrôleurs LED
- Envoi ArtNet vers les contrôleurs physiques
- Optimisé pour les performances

## 🔧 Configuration

### Configuration Principale (`config.toml`)

```toml
[audio]
sample_rate = 48000
buffer_size = 64
gain = 1.5
noise_floor = 0.01

[led]
controllers = [
    "192.168.1.45:6454",
    "192.168.1.46:6454",
    "192.168.1.47:6454",
    "192.168.1.48:6454"
]
fps = 60
brightness = 1.0

[effects]
smoothing_factor = 0.5
bass_boost = 2.5
mid_boost = 2.0
high_boost = 1.5
particle_limit = 2500

[performance]
thread_pool_size = 8
max_cpu_percent = 90.0
```

### Configuration Couleurs Vives

Le projet inclut une configuration optimisée pour des couleurs éclatantes :
- Boost des basses 2.5x
- Réactivité maximale
- 60 FPS
- Luminosité maximale

## 🌐 Interface WebSocket

Le backend expose une interface WebSocket sur `ws://localhost:8080` pour :
- Changer d'effets visuels
- Ajuster les paramètres en temps réel
- Monitorer les performances

### Commandes WebSocket

```json
{
  "type": "change_effect",
  "effect_id": 0
}
```

Effets disponibles :
- `0` : Barres spectrales
- `1` : Ondes circulaires
- `2` : Système de particules

## 📊 Monitoring

Le système affiche en temps réel :
- **FPS LED** : Performance du rendu
- **Niveau audio** : Amplitude du signal
- **Luminosité moyenne** : Intensité des LEDs
- **Statut ArtNet** : Paquets envoyés

## 🎵 Effets Visuels

### Barres Spectrales
- Analyse FFT en temps réel
- Visualisation des fréquences
- Réactif aux basses

### Ondes Circulaires
- Propagation depuis le centre
- Couleurs dynamiques
- Synchronisé au tempo

### Système de Particules
- Particules réactives à l'audio
- Physique temps réel
- Effets de traînée

## 🔌 Contrôleurs LED

### Configuration Réseau
- **Adresses IP** : 192.168.1.45-48
- **Port ArtNet** : 6454
- **Protocole** : ArtNet v4
- **Résolution** : 128x128 pixels par contrôleur

### Mapping des Contrôleurs
```
┌─────────┬─────────┐
│ LED 1   │ LED 2   │
│ .45     │ .46     │
├─────────┼─────────┤
│ LED 3   │ LED 4   │
│ .47     │ .48     │
└─────────┴─────────┘
```

## 🛠️ Développement

### Compilation
```bash
cd apps/backend
cargo build --release
```

### Tests
```bash
cargo test
```

### Développement avec auto-reload
```bash
cargo watch -x run
```

## 📈 Performance

### Optimisations
- **Multithreading** : Capture audio, rendu et réseau séparés
- **SIMD** : Opérations vectorielles pour la FFT
- **Zero-copy** : Minimise les allocations mémoire
- **Buffering intelligent** : Évite les artifacts audio

### Benchmarks Typiques
- **CPU** : 15-30% sur i7
- **RAM** : ~50MB
- **Latence** : <10ms audio vers LED
- **Throughput** : 60 FPS @ 128x128x4 LEDs

## 🐛 Dépannage

### Problèmes Courants

**Pas de contrôleurs détectés**
- Vérifier la connectivité réseau
- Confirmer les adresses IP
- Le mode test se lance automatiquement

**Latence audio élevée**
- Réduire `buffer_size` dans config.toml
- Vérifier la charge CPU
- Utiliser un interface audio dédiée

**Performances dégradées**
- Réduire le `fps` dans la configuration
- Diminuer `particle_limit`
- Activer `frame_skip`

### Logs de Debug

```bash
RUST_LOG=debug ./target/release/led-visualizer
```

## 🔐 Sécurité

- Pas d'authentification (réseau local uniquement)
- Filtrage des commandes WebSocket
- Validation des paramètres de configuration

## 🚧 Roadmap

- [ ] Interface web complète
- [ ] Support MIDI
- [ ] Presets d'effets
- [ ] Enregistrement/replay
- [ ] Support Windows
- [ ] Plugin VST

## 🤝 Contribution

Les contributions sont les bienvenues ! Merci de :
1. Fork le projet
2. Créer une branche feature
3. Committer vos changements
4. Ouvrir une Pull Request

## 📄 License

MIT License - voir le fichier LICENSE pour plus de détails.

## 🙏 Remerciements

- Communauté Rust pour les excellentes librairies
- Protocole ArtNet pour l'éclairage professionnel
- Contributeurs et testeurs

---

**Made with ❤️ for the LED art community**
