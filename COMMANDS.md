# 🚀 Commandes DJ-4LED - Aide-Mémoire

## 🎯 Commandes de Base

### Démarrage Rapide
```bash
# Démarrage backend seul
./start_backend.sh

# Démarrage complet (backend + frontend)
./start_vivid.sh

# Démarrage manuel
cd apps/backend && ./target/release/led-visualizer --production
```

### Arrêt
```bash
# Arrêt propre
Ctrl+C

# Arrêt forcé
pkill led-visualizer
```

## 🔧 Compilation

### Compilation Release
```bash
cd apps/backend
cargo build --release
```

### Compilation Debug
```bash
cargo build
```

### Compilation avec Optimisations
```bash
cargo build --release --target-cpu=native
```

### Nettoyage
```bash
cargo clean
```

## 🧪 Tests et Debug

### Tests
```bash
cargo test
cargo test --release
```

### Lancement avec Logs Debug
```bash
RUST_LOG=debug ./target/release/led-visualizer
RUST_LOG=trace ./target/release/led-visualizer
```

### Profiling
```bash
cargo build --release
perf record ./target/release/led-visualizer
perf report
```

### Vérification Mémoire
```bash
valgrind --tool=memcheck ./target/release/led-visualizer
```

## 📊 Monitoring

### Statistiques CPU/RAM
```bash
top -p $(pgrep led-visualizer)
htop -p $(pgrep led-visualizer)
```

### Trafic Réseau
```bash
# Surveiller le trafic ArtNet
sudo tcpdump -i any port 6454

# Statistiques réseau
netstat -i
```

### WebSocket
```bash
# Tester la connexion WebSocket
wscat -c ws://localhost:8080

# Envoyer une commande
echo '{"type":"change_effect","effect_id":1}' | wscat -c ws://localhost:8080
```

## 🔌 Contrôleurs LED

### Test de Connectivité
```bash
# Ping tous les contrôleurs
for ip in 192.168.1.45 192.168.1.46 192.168.1.47 192.168.1.48; do
    ping -c 1 $ip && echo "✅ $ip OK" || echo "❌ $ip FAIL"
done

# Test port ArtNet
nc -u -z 192.168.1.45 6454
```

### Scan Réseau
```bash
# Trouver tous les contrôleurs sur le réseau
nmap -sU -p 6454 192.168.1.0/24

# Scan rapide
nmap -sn 192.168.1.0/24
```

## 📁 Gestion des Fichiers

### Configuration
```bash
# Sauvegarder config actuelle
cp apps/backend/config.toml config.backup.$(date +%s).toml

# Restaurer config
cp config.backup.toml apps/backend/config.toml

# Appliquer config couleurs vives
cp config.vivid.toml apps/backend/config.toml
```

### Logs
```bash
# Rediriger logs vers fichier
./start_backend.sh > logs/$(date +%Y%m%d_%H%M%S).log 2>&1

# Suivre logs en temps réel
tail -f logs/latest.log
```

## 🌐 Commandes WebSocket

### Changement d'Effets
```bash
# Barres spectrales
echo '{"type":"change_effect","effect_id":0}' | wscat -c ws://localhost:8080

# Ondes circulaires
echo '{"type":"change_effect","effect_id":1}' | wscat -c ws://localhost:8080

# Système de particules
echo '{"type":"change_effect","effect_id":2}' | wscat -c ws://localhost:8080
```

### Paramètres en Temps Réel
```bash
# Changer luminosité
echo '{"type":"set_brightness","value":0.8}' | wscat -c ws://localhost:8080

# Changer gain audio
echo '{"type":"set_gain","value":1.5}' | wscat -c ws://localhost:8080

# Boost des basses
echo '{"type":"set_bass_boost","value":2.0}' | wscat -c ws://localhost:8080
```

## 🔄 Gestion des Processus

### Informations Processus
```bash
# PID et infos
ps aux | grep led-visualizer
pgrep -f led-visualizer

# Arbre des processus
pstree -p $(pgrep led-visualizer)
```

### Redémarrage
```bash
# Redémarrage propre
pkill led-visualizer && sleep 2 && ./start_backend.sh

# Redémarrage forcé
pkill -9 led-visualizer && ./start_backend.sh
```

### Démarrage en Arrière-Plan
```bash
# Démarrage en daemon
nohup ./start_backend.sh > /dev/null 2>&1 &

# Avec logs
nohup ./start_backend.sh > logs/daemon.log 2>&1 &
```

## 🛠️ Développement

### Auto-Recompilation
```bash
# Installer cargo-watch
cargo install cargo-watch

# Auto-recompilation et run
cargo watch -x run
cargo watch -x 'run -- --test'
```

### Benchmark
```bash
# Benchmark intégré
cargo bench

# Benchmark personnalisé
cargo run --release --bin benchmark
```

### Profiling Detaillé
```bash
# Installer perf
sudo apt install linux-perf  # Ubuntu/Debian
brew install perf            # macOS

# Profiling complet
perf record -g ./target/release/led-visualizer
perf report -g
```

## 🐛 Dépannage

### Diagnostic Rapide
```bash
# Vérifier que tout fonctionne
./scripts/health_check.sh

# Test complet du système
./scripts/system_test.sh
```

### Problèmes Audio
```bash
# Lister périphériques audio
pactl list sources        # Linux
system_profiler SPAudioDataType  # macOS

# Tester capture audio
arecord -l               # Linux
rec test.wav             # Test d'enregistrement
```

### Problèmes Réseau
```bash
# Tester routage
traceroute 192.168.1.45
route -n

# Vérifier firewall
sudo ufw status          # Ubuntu
sudo iptables -L         # Linux général
```

## 📦 Installation et Mise à Jour

### Installation Rust
```bash
# Installation Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Mise à jour
rustup update
```

### Dépendances Système
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libasound2-dev

# macOS
brew install pkg-config
```

## 📈 Optimisation

### Optimisation Système
```bash
# Priorité haute pour le processus
sudo nice -n -10 ./target/release/led-visualizer

# Affectation CPU
taskset -c 0,1 ./target/release/led-visualizer
```

### Optimisation Réseau
```bash
# Optimiser buffer réseau
sudo sysctl net.core.rmem_max=268435456
sudo sysctl net.core.wmem_max=268435456
```

## 🔐 Sécurité

### Permissions
```bash
# Vérifier permissions
ls -la apps/backend/target/release/led-visualizer

# Permissions audio (si nécessaire)
sudo usermod -a -G audio $USER
```

### Firewall
```bash
# Ouvrir port WebSocket
sudo ufw allow 8080

# Ouvrir port ArtNet
sudo ufw allow out 6454
```

## 📊 Scripts Utiles

### Script de Monitoring
```bash
#!/bin/bash
# monitor.sh
while true; do
    echo "=== $(date) ==="
    ps aux | grep led-visualizer | head -1
    netstat -i | grep wlan0
    sleep 5
done
```

### Script de Sauvegarde
```bash
#!/bin/bash
# backup.sh
DATE=$(date +%Y%m%d_%H%M%S)
mkdir -p backups
cp -r apps/backend/config.toml backups/config_$DATE.toml
echo "✅ Config sauvegardée dans backups/config_$DATE.toml"
```

---

**💡 Conseil** : Ajoutez ces commandes à votre `.bashrc` ou `.zshrc` pour des aliases pratiques !

```bash
# Aliases DJ-4LED
alias djstart="./start_backend.sh"
alias djstop="pkill led-visualizer"
alias djlogs="tail -f logs/latest.log"
alias djconfig="nano apps/backend/config.toml"
```
