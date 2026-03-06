import { Redirect } from 'expo-router';
import { useAuthStore } from '@/src/stores/auth';

export default function Index() {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const isLoading = useAuthStore((s) => s.isLoading);

  if (isLoading) return null;

  return <Redirect href={isAuthenticated ? '/(tabs)/capture' : '/(auth)/login'} />;
}
