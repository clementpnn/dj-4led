#!/bin/bash

# Script simple pour démarrer uniquement le backend DJ-4LED

echo "🎵 DJ-4LED Backend"
echo "=================="
echo ""

# Couleurs pour l'affichage
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

# Obtenir le répertoire du script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Vérifier la connexion réseau rapidement
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
    echo -e "${YELLOW}⚠️  Aucun contrôleur détecté - Mode test activé${NC}"
    MODE_ARGS="--production --test"
fi

# Vérifier si le port UDP est déjà utilisé
echo -e "${CYAN}🔍 Vérification du port UDP 8081...${NC}"
if lsof -i :8081 > /dev/null 2>&1; then
    echo -e "${RED}❌ Le port UDP 8081 est déjà utilisé${NC}"
    echo "Processus utilisant le port:"
    lsof -i :8081
    echo ""
    echo "Arrêtez le processus existant avec: kill \$(lsof -t -i :8081)"
    exit 1
fi
echo -e "${GREEN}✅ Port UDP 8081 disponible${NC}"

# Trouver le dossier backend
if [ -d "apps/backend" ]; then
    BACKEND_DIR="apps/backend"
elif [ -d "backend" ]; then
    BACKEND_DIR="backend"
else
    echo -e "${RED}❌ Impossible de trouver le dossier backend${NC}"
    exit 1
fi

# Compiler si nécessaire
echo -e "${CYAN}🔨 Compilation du backend...${NC}"
cd "$BACKEND_DIR"

if [ ! -f "target/release/led-visualizer" ] || [ "src/main.rs" -nt "target/release/led-visualizer" ]; then
    echo "   Compilation en cours..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Erreur lors de la compilation${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Compilation terminée${NC}"
else
    echo -e "${GREEN}✅ Binaire déjà compilé${NC}"
fi

# Copier la configuration vivid si elle existe
if [ -f "../config.vivid.toml" ]; then
    cp "../config.vivid.toml" config.toml
    echo -e "${GREEN}✅ Configuration couleurs vives appliquée${NC}"
elif [ -f "../../config.vivid.toml" ]; then
    cp "../../config.vivid.toml" config.toml
    echo -e "${GREEN}✅ Configuration couleurs vives appliquée${NC}"
fi

# Fonction de nettoyage
cleanup() {
    echo ""
    echo -e "${CYAN}🛑 Arrêt du backend...${NC}"
    if [ ! -z "$BACKEND_PID" ]; then
        kill $BACKEND_PID 2>/dev/null
        wait $BACKEND_PID 2>/dev/null
    fi
    exit 0
}

trap cleanup INT TERM

# Démarrer le backend
echo ""
echo -e "${CYAN}🎵 Démarrage du backend...${NC}"
echo -e "${CYAN}🔌 Serveur UDP sur :${NC} udp://0.0.0.0:8081"
echo ""

./target/release/led-visualizer $MODE_ARGS &
BACKEND_PID=$!

# Vérifier que le backend a bien démarré
sleep 2
if ! kill -0 $BACKEND_PID 2>/dev/null; then
    echo -e "${RED}❌ Le backend n'a pas pu démarrer${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Backend démarré avec succès !${NC}"
echo ""
echo "========================================"
echo -e "${YELLOW}💡 Informations :${NC}"
echo "   • Serveur UDP : udp://0.0.0.0:8081"
echo "   • Mode : $MODE_ARGS"
echo "   • PID : $BACKEND_PID"
echo ""
echo -e "${CYAN}📊 Commandes :${NC}"
echo "   • Ctrl+C pour arrêter"
echo "   • Logs en temps réel ci-dessous"
echo ""
echo "========================================"

if [[ "$MODE_ARGS" == *"--test"* ]]; then
    echo -e "${YELLOW}⚠️  Mode test audio (simulation)${NC}"
    echo ""
fi

# Attendre que le backend se termine
wait $BACKEND_PID
