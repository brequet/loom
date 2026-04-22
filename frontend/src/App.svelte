<script lang="ts">
  import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
  import Router from 'svelte-spa-router';
  import Layout from '$lib/components/Layout.svelte';
  import Dashboard from '$lib/pages/Dashboard.svelte';
  import SessionDetail from '$lib/pages/SessionDetail.svelte';
  import Settings from '$lib/pages/Settings.svelte';

  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 5_000,
        refetchOnWindowFocus: true,
        retry: 1,
      },
    },
  });

  const routes = {
    '/': Dashboard,
    '/sessions/:id': SessionDetail,
    '/settings': Settings,
  };
</script>

<QueryClientProvider client={queryClient}>
  <Layout>
    <Router {routes} />
  </Layout>
</QueryClientProvider>
