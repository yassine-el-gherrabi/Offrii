import { View, StyleSheet } from 'react-native';
import { Text } from 'react-native-paper';
import { useTranslation } from 'react-i18next';

export default function ListScreen() {
  const { t } = useTranslation();

  return (
    <View style={styles.container}>
      <Text variant="headlineMedium">{t('list.title')}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
});
