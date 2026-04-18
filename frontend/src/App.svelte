<script lang="ts">
  import './app.css';
  import Router from 'svelte-spa-router';
  import Layout from '$lib/components/Layout.svelte';
  import Dashboard from '$lib/pages/Dashboard.svelte';
  import SessionDetail from '$lib/pages/SessionDetail.svelte';
  import Settings from '$lib/pages/Settings.svelte';

  const routes = {
    '/': Dashboard,
    '/sessions/:id': SessionDetail,
    '/settings': Settings,
  };
</script>

<Layout>
  <Router {routes} />
</Layout>
