import { View, StyleSheet } from 'react-native';
import { Text } from 'react-native-paper';

export default function RegisterScreen() {
  return (
    <View style={styles.container}>
      <Text variant="headlineMedium">Inscription</Text>
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
