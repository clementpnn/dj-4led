# 🔧 Exemples de Configuration - DJ-4LED

## 🎯 Configurations Prêtes à l'Emploi

### 🌈 Configuration Couleurs Vives (config.vivid.toml)
```toml
# Configuration pour des couleurs vives et saturées
[audio]
sample_rate = 48000
buffer_size = 32    # Ultra-réactif
gain = 2.0          # Gain élevé
noise_floor = 0.005 # Seuil très bas

[led]
controllers = [
    "192.168.1.45:6454",
    "192.168.1.46:6454",
    "192.168.1.47:6454",
    "192.168.1.48:6454",
]
fps = 60
brightness = 1.0
gamma_correction = 1.0  # Couleurs pures
color_temperature = 1.0

[effects]
smoothing_factor = 0.3  # Peu de lissage
bass_boost = 3.0        # Boost maximal des basses
mid_boost = 2.2
high_boost = 1.8
particle_limit = 3000   # Maximum de particules
wave_speed = 2.0        # Ondes rapides

[performance]
thread_pool_size = 12
frame_skip = false
adaptive_quality = false
max_cpu_percent = 95.0
```

### 🎵 Configuration Concert (config.concert.toml)
```toml
# Optimisé pour les concerts et événements live
[audio]
sample_rate = 44100
buffer_size = 64
gain = 1.5
noise_floor = 0.02

[led]
controllers = [
    "192.168.1.45:6454",
    "192.168.1.46:6454",
    "192.168.1.47:6454",
    "192.168.1.48:6454",
]
fps = 50            # FPS stable pour la vidéo
brightness = 0.9    # Légèrement réduit pour éviter l'éblouissement
gamma_correction = 2.2
color_temperature = 1.0

[effects]
smoothing_factor = 0.6  # Plus de lissage pour la stabilité
bass_boost = 2.0
mid_boost = 1.5
high_boost = 1.2
particle_limit = 2000
wave_speed = 1.5

[performance]
thread_pool_size = 8
frame_skip = true       # Permet de skipper si nécessaire
adaptive_quality = true
max_cpu_percent = 75.0  # Laisse de la marge pour autres processus
```

### 🏠 Configuration Maison (config.home.toml)
```toml
# Pour utilisation domestique et développement
[audio]
sample_rate = 44100
buffer_size = 128
gain = 1.0
noise_floor = 0.03

[led]
controllers = [
    "192.168.1.45:6454",
]
fps = 30            # FPS réduit pour économie d'énergie
brightness = 0.7    # Luminosité confortable
gamma_correction = 2.2
color_temperature = 0.9  # Légèrement chaud

[effects]
smoothing_factor = 0.8  # Très lissé pour ambiance douce
bass_boost = 1.2
mid_boost = 1.0
high_boost = 0.8
particle_limit = 1000
wave_speed = 0.8

[performance]
thread_pool_size = 4
frame_skip = true
adaptive_quality = true
max_cpu_percent = 50.0  # Utilisation CPU modérée
```

### 🧪 Configuration Test (config.test.toml)
```toml
# Pour tests et développement
[audio]
sample_rate = 44100
buffer_size = 256      # Buffer plus grand pour stabilité
gain = 1.0
noise_floor = 0.05

[led]
controllers = []       # Pas de contrôleurs physiques
fps = 20               # FPS bas pour économie
brightness = 0.5
gamma_correction = 1.0
color_temperature = 1.0

[effects]
smoothing_factor = 0.5
bass_boost = 1.0
mid_boost = 1.0
high_boost = 1.0
particle_limit = 500   # Peu de particules
wave_speed = 1.0

[performance]
thread_pool_size = 2
frame_skip = true
adaptive_quality = true
max_cpu_percent = 30.0
```

### 🚀 Configuration Haute Performance (config.highperf.toml)
```toml
# Pour systèmes puissants et installations permanentes
[audio]
sample_rate = 96000    # Très haute qualité
buffer_size = 16       # Buffer minimal
gain = 1.8
noise_floor = 0.001    # Seuil très bas

[led]
controllers = [
    "192.168.1.45:6454",
    "192.168.1.46:6454",
    "192.168.1.47:6454",
    "192.168.1.48:6454",
    "192.168.1.49:6454",
    "192.168.1.50:6454",
    "192.168.1.51:6454",
    "192.168.1.52:6454",
]
fps = 120              # FPS très élevé
brightness = 1.0
gamma_correction = 1.8
color_temperature = 1.0

[effects]
smoothing_factor = 0.2  # Très réactif
bass_boost = 4.0        # Boost extrême
mid_boost = 3.0
high_boost = 2.5
particle_limit = 5000   # Maximum de particules
wave_speed = 3.0        # Très rapide

[performance]
thread_pool_size = 16   # Utilise tous les cœurs
frame_skip = false
adaptive_quality = false
max_cpu_percent = 98.0  # Utilisation maximale
```

## 🎨 Configurations par Genre Musical

### 🎸 Rock/Metal
```toml
[effects]
bass_boost = 3.5       # Basses très présentes
mid_boost = 2.0        # Guitares
high_boost = 1.5       # Cymbales
smoothing_factor = 0.2 # Très réactif aux changements
wave_speed = 2.5       # Rapide et énergique
```

### 🎹 Jazz/Blues
```toml
[effects]
bass_boost = 1.8       # Basses présentes mais douces
mid_boost = 1.5        # Piano/sax
high_boost = 1.2       # Subtil
smoothing_factor = 0.7 # Lissé pour fluidité
wave_speed = 1.0       # Tempo modéré
```

### 🎧 Électronique/EDM
```toml
[effects]
bass_boost = 4.0       # Basses très puissantes
mid_boost = 2.5        # Synthés
high_boost = 2.0       # Effets high-freq
smoothing_factor = 0.1 # Très réactif
wave_speed = 3.0       # Très rapide
particle_limit = 4000  # Beaucoup d'effets
```

### 🎼 Classique
```toml
[effects]
bass_boost = 1.2       # Basses subtiles
mid_boost = 1.0        # Équilibré
high_boost = 1.0       # Naturel
smoothing_factor = 0.9 # Très lissé
wave_speed = 0.5       # Lent et majestueux
```

## 🌐 Configurations Réseau

### 🏢 Installation Professionnelle
```toml
[led]
controllers = [
    "10.0.1.10:6454",   # Scène principale
    "10.0.1.11:6454",   # Scène secondaire
    "10.0.1.20:6454",   # Éclairage public
    "10.0.1.21:6454",   # Éclairage d'ambiance
]
```

### 🏠 Réseau Domestique
```toml
[led]
controllers = [
    "192.168.1.100:6454",  # Salon
    "192.168.1.101:6454",  # Chambre
]
```

### 🧪 Tests Locaux
```toml
[led]
controllers = [
    "127.0.0.1:6454",     # Simulateur local
]
```

## 🛠️ Utilisation des Configurations

### Appliquer une Configuration
```bash
# Copier la configuration désirée
cp examples/config.vivid.toml apps/backend/config.toml

# Redémarrer le backend
./start_backend.sh
```

### Créer sa Propre Configuration
```bash
# Partir d'un exemple
cp examples/config.home.toml my-config.toml

# Éditer selon ses besoins
nano my-config.toml

# Appliquer
cp my-config.toml apps/backend/config.toml
```

## 📊 Paramètres Critiques

### Pour la Performance
- `fps` : 30-60 pour usage normal, 120+ pour haute performance
- `thread_pool_size` : Nombre de cœurs CPU disponibles
- `buffer_size` : Plus petit = plus réactif, plus grand = plus stable

### Pour la Qualité Visuelle
- `brightness` : 0.5-1.0 selon l'environnement
- `gamma_correction` : 1.0 pour couleurs pures, 2.2 pour naturel
- `smoothing_factor` : 0.1-0.9 selon le style désiré

### Pour la Réactivité Audio
- `gain` : 1.0-2.0 selon la source audio
- `noise_floor` : 0.001-0.05 selon l'environnement
- `bass_boost` : 1.0-4.0 selon le genre musical

## 🔄 Changement de Configuration à Chaud

```bash
# Sauvegarder la configuration actuelle
cp apps/backend/config.toml config.backup.toml

# Appliquer une nouvelle configuration
cp examples/config.concert.toml apps/backend/config.toml

# Redémarrer pour appliquer
pkill led-visualizer
./start_backend.sh
```

---

**💡 Conseil** : Commencez par une configuration de base et ajustez progressivement selon vos besoins !
