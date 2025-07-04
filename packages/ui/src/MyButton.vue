<script setup lang="ts">
    defineProps<{ onClick?: () => void }>()
</script>

<template>
    <button class="btn"><slot /></button>
</template>