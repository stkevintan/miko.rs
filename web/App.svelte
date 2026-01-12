<script lang="ts">
    import Router from 'svelte-spa-router';
    import Login from './routes/Login.svelte';
    import Dashboard from './routes/Dashboard.svelte';

    const routes = {
        '/': Login,
        '/login': Login,
        '/dashboard': Dashboard,
        '*': Login,
    };
</script>

<Router {routes} />

<style lang="scss">
    /* SCSS power here */
</style>
