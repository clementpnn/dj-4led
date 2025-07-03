#!/bin/bash

# Script de lancement rapide DJ-4LED avec couleurs vives
# Lance le backend et le frontend avec des paramètres optimisés pour des couleurs éclatantes

echo "🌈 DJ-4LED - Mode Couleurs Vives"
echo "================================"
echo ""

# Couleurs pour l'affichage
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Obtenir le répertoire du script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Vérifier la connexion réseau rapidement
echo -e "${CYAN}📡 Vérification rapide du réseau...${NC}"
CONTROLLERS_OK=0
for ip in 192.168.1.45 192.168.1.46 192.168.1.47 192.168.1.48; do
    if ping -c 1 -W 1 $ip > /dev/null 2>&1; then
        ((CONTROLLERS_OK++))
    fi
done

if [ $CONTROLLERS_OK -gt 0 ]; then
    echo -e "${GREEN}✅ $CONTROLLERS_OK/4 contrôleurs détectés${NC}"
    TEST_MODE=""
else
    echo -e "${YELLOW}⚠️  Aucun contrôleur détecté - Mode test activé${NC}"
    TEST_MODE="--test"
fi

# Vérifier que le dossier backend existe
if [ ! -d "apps/backend" ]; then
    echo -e "${YELLOW}⚠️  Dossier apps/backend non trouvé${NC}"
    echo "   Tentative avec le dossier backend..."
    if [ -d "backend" ]; then
        BACKEND_DIR="backend"
    else
        echo "❌ Impossible de trouver le dossier backend"
        exit 1
    fi
else
    BACKEND_DIR="apps/backend"
fi

# Compiler si nécessaire
echo -e "${CYAN}🔨 Vérification du backend...${NC}"
cd "$BACKEND_DIR"
if [ ! -f "target/release/led-visualizer" ] || [ "src/main.rs" -nt "target/release/led-visualizer" ]; then
    echo "   Compilation en cours..."
    cargo build --release --quiet
    if [ $? -ne 0 ]; then
        echo "❌ Erreur lors de la compilation"
        exit 1
    fi
fi
cd "$SCRIPT_DIR"

# Copier la configuration vivid
if [ -f "config.vivid.toml" ]; then
    cp config.vivid.toml "$BACKEND_DIR/config.toml"
    echo -e "${GREEN}✅ Configuration couleurs vives appliquée${NC}"
fi

# Variables pour les PID
BACKEND_PID=""
FRONTEND_PID=""

# Fonction de nettoyage
cleanup() {
    echo ""
    echo -e "${CYAN}🛑 Arrêt...${NC}"
    if [ ! -z "$BACKEND_PID" ]; then
        kill $BACKEND_PID 2>/dev/null
        wait $BACKEND_PID 2>/dev/null
    fi
    if [ ! -z "$FRONTEND_PID" ]; then
        kill $FRONTEND_PID 2>/dev/null
        wait $FRONTEND_PID 2>/dev/null
    fi
    exit 0
}

trap cleanup INT TERM

# Démarrer le backend
echo -e "${CYAN}🎵 Démarrage du backend...${NC}"
cd "$BACKEND_DIR"

# Construire la commande
if [ -z "$TEST_MODE" ]; then
    ./target/release/led-visualizer --production &
else
    ./target/release/led-visualizer --production $TEST_MODE &
fi

BACKEND_PID=$!
cd "$SCRIPT_DIR"

# Vérifier que le backend a bien démarré
sleep 2
if ! kill -0 $BACKEND_PID 2>/dev/null; then
    echo "❌ Le backend n'a pas pu démarrer"
    exit 1
fi

# Démarrer le frontend (décommenté)
echo -e "${CYAN}🌐 Démarrage de l'interface web...${NC}"
if [ -d "apps/frontend" ]; then
    cd apps/frontend
    FRONTEND_DIR="apps/frontend"
elif [ -d "frontend" ]; then
    cd frontend
    FRONTEND_DIR="frontend"
else
    echo -e "${YELLOW}⚠️  Dossier frontend non trouvé, backend seul${NC}"
    FRONTEND_DIR=""
fi

if [ ! -z "$FRONTEND_DIR" ]; then
    # Installation rapide des dépendances si nécessaire
    if [ ! -d "node_modules" ]; then
        echo "   Installation des dépendances..."
        npm install --silent
    fi

    # Lancer le frontend en mode silencieux
    npm run dev --silent > /dev/null 2>&1 &
    FRONTEND_PID=$!
    cd "$SCRIPT_DIR"

    # Attendre que le frontend démarre
    sleep 3
fi

# Afficher les infos
echo ""
echo "========================================"
echo -e "${GREEN}✨ DJ-4LED opérationnel !${NC}"
echo ""
if [ ! -z "$FRONTEND_DIR" ]; then
    echo -e "${CYAN}🎮 Interface web :${NC} http://localhost:5173"
fi
echo -e "${CYAN}🔌 WebSocket :${NC} ws://localhost:8080"
echo ""
echo -e "${YELLOW}💡 Configuration appliquée :${NC}"
echo "   • Couleurs vives et saturées"
echo "   • Boost des basses : 2.5x"
echo "   • Réactivité maximale"
echo "   • 60 FPS"
echo ""
echo -e "${CYAN}📊 Commandes :${NC}"
echo "   • Changer d'effet via l'interface web"
echo "   • Ctrl+C pour arrêter"
echo ""

if [ ! -z "$TEST_MODE" ]; then
    echo -e "${YELLOW}⚠️  Mode test audio (pas de contrôleurs détectés)${NC}"
fi

# Ouvrir automatiquement le navigateur après 2 secondes (macOS/Linux)
if [ ! -z "$FRONTEND_DIR" ]; then
    if command -v open > /dev/null 2>&1; then
        (sleep 2 && open http://localhost:5173) &
    elif command -v xdg-open > /dev/null 2>&1; then
        (sleep 2 && xdg-open http://localhost:5173) &
    fi
fi

# Attendre que le backend se termine
wait $BACKEND_PID
