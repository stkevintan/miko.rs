<script lang="ts">
    import Router from 'svelte-spa-router';
    import Login from './routes/Login.svelte';
    import Dashboard from './routes/Dashboard.svelte';
    import Library from './routes/Library.svelte';
    import Toast from './components/ui/Toast.svelte';

    const routes = {
        '/': Dashboard,
        '/login': Login,
        '/dashboard': Dashboard,
        '/library': Library,
        '*': Dashboard,
    };
</script>

<Router {routes} />
<Toast />

<style lang="scss">
    /* SCSS power here */
</style>
