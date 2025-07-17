#!/bin/bash

# Script simple pour démarrer le backend DJ-4LED

echo "🎵 DJ-4LED Backend"
echo "=================="
echo ""

# Couleurs
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

# Variables
BACKEND_PID=""
FORCE_BUILD=false

# Vérifier les arguments
if [[ "$1" == "--force" || "$1" == "-f" ]]; then
    FORCE_BUILD=true
    echo -e "${YELLOW}🔄 Build forcé activé${NC}"
fi

# Aller dans le bon répertoire
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Vérifier les contrôleurs
echo -e "${CYAN}📡 Vérification des contrôleurs...${NC}"
CONTROLLERS_OK=0
for ip in 192.168.1.45 192.168.1.46 192.168.1.47 192.168.1.48; do
    if ping -c 1 -W 1 $ip > /dev/null 2>&1; then
        ((CONTROLLERS_OK++))
    fi
done

if [ $CONTROLLERS_OK -gt 0 ]; then
    echo -e "${GREEN}✅ $CONTROLLERS_OK/4 contrôleurs détectés${NC}"
    MODE_ARGS="--production"
else
    echo -e "${YELLOW}⚠️  Mode test activé${NC}"
    MODE_ARGS="--production --test"
fi

# Trouver le dossier backend
if [ -d "apps/backend" ]; then
    BACKEND_DIR="apps/backend"
elif [ -d "backend" ]; then
    BACKEND_DIR="backend"
else
    echo -e "${RED}❌ Dossier backend introuvable${NC}"
    exit 1
fi

cd "$BACKEND_DIR"
echo -e "${CYAN}📂 Backend: $BACKEND_DIR${NC}"

# Compilation
echo -e "${CYAN}🔨 Compilation...${NC}"

# Forcer le build si demandé
if [ "$FORCE_BUILD" = true ]; then
    echo "   Nettoyage..."
    cargo clean
fi

# Compiler si nécessaire
if [ "$FORCE_BUILD" = true ] || [ ! -f "target/release/led-visualizer" ] || [ "src/main.rs" -nt "target/release/led-visualizer" ]; then
    echo "   Building..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Erreur compilation${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Compilé${NC}"
else
    echo -e "${GREEN}✅ Déjà compilé${NC}"
fi

# Copier config si elle existe
for config in "../config.vivid.toml" "../../config.vivid.toml"; do
    if [ -f "$config" ]; then
        cp "$config" config.toml
        echo -e "${GREEN}✅ Config appliquée${NC}"
        break
    fi
done

# Fonction de nettoyage
cleanup() {
    echo ""
    echo -e "${CYAN}🛑 Arrêt...${NC}"
    if [ ! -z "$BACKEND_PID" ]; then
        kill $BACKEND_PID 2>/dev/null
        wait $BACKEND_PID 2>/dev/null
    fi
    exit 0
}

trap cleanup INT TERM

# Démarrer
echo ""
echo -e "${CYAN}🎵 Démarrage backend...${NC}"
echo -e "${CYAN}🔌 WebSocket: ws://localhost:8080${NC}"
echo ""

./target/release/led-visualizer $MODE_ARGS &
BACKEND_PID=$!

# Vérifier le démarrage
sleep 2
if ! kill -0 $BACKEND_PID 2>/dev/null; then
    echo -e "${RED}❌ Échec démarrage${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Backend démarré !${NC}"
echo "PID: $BACKEND_PID"
echo "Ctrl+C pour arrêter"
echo ""

if [[ "$MODE_ARGS" == *"--test"* ]]; then
    echo -e "${YELLOW}⚠️  Mode test${NC}"
fi

# Attendre
wait $BACKEND_PID
