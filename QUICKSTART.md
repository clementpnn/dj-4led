# 🚀 Guide de Démarrage Rapide - DJ-4LED

## En 30 secondes

```bash
# 1. Cloner et entrer dans le projet
git clone <repository-url>
cd dj-4led

# 2. Rendre les scripts exécutables
chmod +x *.sh

# 3. Démarrer le système
./start_backend.sh
```

## 🎯 Démarrage Immédiat

### Option 1 : Backend seul (recommandé)
```bash
./start_backend.sh
```
✅ **Résultat** : Backend démarré sur `ws://localhost:8080`

### Option 2 : Backend + Interface Web
```bash
./start_vivid.sh
```
✅ **Résultat** : Interface web sur `http://localhost:5173`

## 🔧 Que font les scripts ?

### `start_backend.sh`
- 🔍 Détecte automatiquement les contrôleurs LED
- 🔨 Compile le backend si nécessaire
- 🎵 Lance en mode test si aucun contrôleur trouvé
- 📊 Affiche les statistiques temps réel

### `start_vivid.sh`
- 🌈 Mode couleurs vives optimisé
- 🌐 Interface web incluse
- 🎮 Ouverture automatique du navigateur
- 🔄 Gestion complète des processus

## 🚦 Statuts de Démarrage

### ✅ Succès
```
🎵 DJ-4LED Backend
==================
✅ 2/4 contrôleurs détectés
✅ Compilation terminée
✅ Backend démarré avec succès !
```

### ⚠️ Mode Test
```
⚠️ Aucun contrôleur détecté - Mode test activé
🧪 Mode test audio (simulation)
```

### ❌ Erreur
```
❌ Impossible de trouver le dossier backend
❌ Erreur lors de la compilation
```

## 🎮 Contrôles

### Via Terminal
- `Ctrl+C` : Arrêter le système
- Les logs s'affichent en temps réel

### Via WebSocket (port 8080)
```json
{"type": "change_effect", "effect_id": 0}
```

## 🌐 Adresses Importantes

- **Backend WebSocket** : `ws://localhost:8080`
- **Interface Web** : `http://localhost:5173`
- **Contrôleurs LED** : `192.168.1.45-48:6454`

## 🛠️ Dépannage Express

### Problème : "Dossier backend non trouvé"
```bash
# Vérifier la structure
ls -la
# Doit contenir : apps/backend/ ou backend/
```

### Problème : "Erreur de compilation"
```bash
# Installer Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Problème : "Aucun contrôleur détecté"
```bash
# Tester la connectivité
ping 192.168.1.45
# Le mode test se lance automatiquement
```

## 🏃‍♂️ Démarrage Manuel (Avancé)

```bash
# 1. Aller dans le backend
cd apps/backend

# 2. Compiler
cargo build --release

# 3. Lancer avec options
./target/release/led-visualizer --production        # Mode production
./target/release/led-visualizer --test              # Mode test
./target/release/led-visualizer --production --test # Les deux
```

## 📊 Vérification du Fonctionnement

### Logs Normaux
```
🎵 [SpectrumBars] Audio level: 0.47, spectrum[0]: 0.46
📡 Sending frame - avg brightness: 22.7
✅ Sent 128 ArtNet packets
📊 LED FPS: 59.4 | Frames: 100
```

### Indicateurs de Performance
- **Audio level** : 0.0 à 1.0 (niveau sonore)
- **Brightness** : 0 à 100 (luminosité moyenne)
- **LED FPS** : Performance du rendu
- **ArtNet packets** : Communication avec les contrôleurs

## 🎵 Test Audio

Le système capture automatiquement l'audio :
- **macOS** : Audio système par défaut
- **Mode test** : Audio simulé pour les tests
- **Pas de son** : Vérifier les permissions micro

## 🔄 Redémarrage Rapide

```bash
# Arrêter (Ctrl+C puis)
./start_backend.sh  # Redémarrer
```

## 💡 Conseils Pro

1. **Première utilisation** : Commencer par `./start_backend.sh`
2. **Développement** : Utiliser le mode test
3. **Production** : Vérifier la connectivité réseau
4. **Performance** : Surveiller les FPS et la charge CPU

## 🆘 Aide Rapide

| Problème | Solution |
|----------|----------|
| Pas de son | Vérifier les permissions audio |
| Lag/FPS bas | Réduire `fps` dans config.toml |
| Crash | Vérifier les logs avec `RUST_LOG=debug` |
| Réseau | Vérifier les IP des contrôleurs |

---

**🎉 Félicitations ! Votre système DJ-4LED est opérationnel !**

Pour plus d'informations, consultez le [README.md](README.md) complet.
