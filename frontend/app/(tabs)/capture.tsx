import { View, StyleSheet } from 'react-native';
import { Text } from 'react-native-paper';

export default function CaptureScreen() {
  return (
    <View style={styles.container}>
      <Text variant="headlineMedium">Capturer une offre</Text>
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
