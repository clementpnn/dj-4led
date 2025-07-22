#!/bin/bash

# DJ-4LED - Version robuste avec reconnexion automatique

echo "🔄 DJ-4LED - Version Robuste"
echo "============================"

# Couleurs
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Variables
BACKEND_PID=""
FRONTEND_PID=""
MONITOR_PID=""
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESTART_COUNT=0
MAX_RESTARTS=10

cd "$SCRIPT_DIR"

# Détection de structure
BACKEND_DIR="."
FRONTEND_DIR="frontend"

if [ -d "apps/backend" ]; then
    BACKEND_DIR="apps/backend"
    FRONTEND_DIR="apps/frontend"
elif [ -d "backend" ]; then
    BACKEND_DIR="backend"
    FRONTEND_DIR="frontend"
elif [ -d "src-tauri" ]; then
    BACKEND_DIR="src-tauri"
    FRONTEND_DIR="."
fi

echo -e "${CYAN}Backend: $BACKEND_DIR${NC}"
echo -e "${CYAN}Frontend: $FRONTEND_DIR${NC}"

# Test contrôleurs LED plus robuste
echo -e "${CYAN}Test des contrôleurs LED...${NC}"
CONTROLLERS_OK=0

# Force production mode si demandé
if [[ "$1" == "--force-production" || "$1" == "-p" ]]; then
    echo -e "${YELLOW}🔧 Mode production FORCÉ par argument${NC}"
    CONTROLLERS_OK=1
else
    # Test des contrôleurs
    for ip in 192.168.1.45 192.168.1.46 192.168.1.47 192.168.1.48; do
        echo -n "  Teste $ip... "

        # Test spécifique Windows/Git Bash
        if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
            if ping -n 1 -w 1000 $ip >/dev/null 2>&1; then
                echo -e "${GREEN}OK${NC}"
                ((CONTROLLERS_OK++))
            else
                echo -e "${RED}FAIL${NC}"
            fi
        else
            # Linux/Mac
            if ping -c 1 -W 1 $ip >/dev/null 2>&1; then
                echo -e "${GREEN}OK${NC}"
                ((CONTROLLERS_OK++))
            else
                echo -e "${RED}FAIL${NC}"
            fi
        fi
    done
fi

echo ""
if [ $CONTROLLERS_OK -gt 0 ]; then
    echo -e "${GREEN}✅ MODE PRODUCTION ACTIVÉ${NC}"
    BACKEND_ARGS="--production"
    MODE_NAME="PRODUCTION"
else
    echo -e "${YELLOW}⚠️ MODE SIMULATEUR${NC}"
    echo -e "${CYAN}   Conseil: utilisez './start_robust.sh --force-production' pour forcer${NC}"
    BACKEND_ARGS=""
    MODE_NAME="SIMULATEUR"
fi

# Build backend
echo -e "${CYAN}Build du backend...${NC}"
cd "$BACKEND_DIR"

BINARY_NAME="led-visualizer"
if [ -f "Cargo.toml" ]; then
    TOML_NAME=$(grep -E "^name\s*=" Cargo.toml | head -1 | sed 's/.*=\s*"\([^"]*\)".*/\1/' | tr '-' '_')
    if [ ! -z "$TOML_NAME" ]; then
        BINARY_NAME="$TOML_NAME"
    fi
fi

if [ ! -f "target/release/$BINARY_NAME" ] || [ "src/main.rs" -nt "target/release/$BINARY_NAME" ]; then
    echo "  Compilation..."
    cargo build --release --quiet
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Erreur compilation${NC}"
        exit 1
    fi
fi

if [ ! -f "target/release/$BINARY_NAME" ]; then
    echo -e "${RED}❌ Binaire non trouvé: target/release/$BINARY_NAME${NC}"
    exit 1
fi

cd "$SCRIPT_DIR"

# Préparer frontend
echo -e "${CYAN}Préparation du frontend...${NC}"
cd "$FRONTEND_DIR"

if [ ! -d "node_modules" ] || [ ! -f "node_modules/.pnpm/lock.yaml" 2>/dev/null ]; then
    echo "  Installation des dépendances..."
    pnpm install --silent
fi

cd "$SCRIPT_DIR"

# Fonction de démarrage du backend
start_backend() {
    echo -e "${BLUE}🚀 Démarrage backend (tentative $((RESTART_COUNT + 1)))...${NC}"
    cd "$BACKEND_DIR"

    ./target/release/$BINARY_NAME $BACKEND_ARGS &
    BACKEND_PID=$!

    cd "$SCRIPT_DIR"

    # Attendre et vérifier le démarrage
    sleep 3
    if ! kill -0 $BACKEND_PID 2>/dev/null; then
        echo -e "${RED}❌ Backend échec au démarrage${NC}"
        return 1
    fi

    echo -e "${GREEN}✅ Backend démarré (PID: $BACKEND_PID, Mode: $MODE_NAME)${NC}"
    return 0
}

# Fonction de démarrage du frontend
start_frontend() {
    echo -e "${BLUE}🎨 Démarrage frontend...${NC}"
    cd "$FRONTEND_DIR"

    pnpm run tauri dev >/dev/null 2>&1 &
    FRONTEND_PID=$!

    cd "$SCRIPT_DIR"

    # Attendre et vérifier
    sleep 5
    if ! kill -0 $FRONTEND_PID 2>/dev/null; then
        echo -e "${RED}❌ Frontend échec au démarrage${NC}"
        return 1
    fi

    echo -e "${GREEN}✅ Frontend démarré (PID: $FRONTEND_PID)${NC}"
    return 0
}

# Fonction de monitoring et reconnexion
monitor_processes() {
    while true; do
        sleep 10

        # Vérifier backend
        if [ ! -z "$BACKEND_PID" ] && ! kill -0 $BACKEND_PID 2>/dev/null; then
            echo -e "${RED}🔄 Backend arrêté - Redémarrage...${NC}"
            RESTART_COUNT=$((RESTART_COUNT + 1))

            if [ $RESTART_COUNT -gt $MAX_RESTARTS ]; then
                echo -e "${RED}❌ Trop de redémarrages ($MAX_RESTARTS), abandon${NC}"
                break
            fi

            start_backend
            sleep 2
        fi

        # Vérifier frontend
        if [ ! -z "$FRONTEND_PID" ] && ! kill -0 $FRONTEND_PID 2>/dev/null; then
            echo -e "${RED}🔄 Frontend arrêté - Redémarrage...${NC}"
            start_frontend
            sleep 2
        fi

        # Statistiques périodiques
        if [ $(($(date +%s) % 60)) -eq 0 ]; then
            echo -e "${BLUE}💓 Status: Backend PID $BACKEND_PID, Frontend PID $FRONTEND_PID, Redémarrages: $RESTART_COUNT${NC}"
        fi
    done
}

# Nettoyage
cleanup() {
    echo ""
    echo -e "${CYAN}🛑 Arrêt en cours...${NC}"

    [ ! -z "$MONITOR_PID" ] && kill $MONITOR_PID 2>/dev/null
    [ ! -z "$FRONTEND_PID" ] && kill $FRONTEND_PID 2>/dev/null
    [ ! -z "$BACKEND_PID" ] && kill $BACKEND_PID 2>/dev/null

    # Attendre un peu
    sleep 2

    # Force kill si nécessaire
    [ ! -z "$FRONTEND_PID" ] && kill -9 $FRONTEND_PID 2>/dev/null
    [ ! -z "$BACKEND_PID" ] && kill -9 $BACKEND_PID 2>/dev/null

    echo -e "${GREEN}✅ Arrêt terminé${NC}"
    exit 0
}

trap cleanup INT TERM

# Démarrage initial
echo ""
echo -e "${CYAN}======================================${NC}"
echo -e "${CYAN}🚀 DÉMARRAGE DE DJ-4LED${NC}"
echo -e "${CYAN}======================================${NC}"

# Démarrer backend
if ! start_backend; then
    echo -e "${RED}❌ Impossible de démarrer le backend${NC}"
    exit 1
fi

# Démarrer frontend
if ! start_frontend; then
    echo -e "${RED}❌ Impossible de démarrer le frontend${NC}"
    cleanup
    exit 1
fi

# Démarrer le monitoring
echo -e "${BLUE}🔍 Démarrage du monitoring automatique...${NC}"
monitor_processes &
MONITOR_PID=$!

# Status final
echo ""
echo -e "${GREEN}🎉 DJ-4LED DÉMARRÉ AVEC SUCCÈS !${NC}"
echo "═══════════════════════════════════════════════"
echo -e "${CYAN}Backend:${NC}     PID $BACKEND_PID (Mode: $MODE_NAME)"
echo -e "${CYAN}Frontend:${NC}    PID $FRONTEND_PID"
echo -e "${CYAN}Monitor:${NC}     PID $MONITOR_PID (Auto-restart activé)"
echo -e "${CYAN}WebSocket:${NC}   ws://localhost:8080"
echo ""
echo -e "${YELLOW}💡 Reconnexion automatique activée${NC}"
echo -e "${YELLOW}💡 Maximum $MAX_RESTARTS redémarrages${NC}"
echo -e "${YELLOW}💡 Ctrl+C pour tout arrêter${NC}"
echo ""

# Attendre
wait $MONITOR_PID
