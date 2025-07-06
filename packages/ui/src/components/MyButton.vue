<script setup lang="ts">
    defineProps<{ onClick?: () => void }>()
</script>

<template>
    <button class="btn" @click="onClick">
        <slot />
    </button>
</template>