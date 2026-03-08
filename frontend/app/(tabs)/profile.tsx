import { useCallback, useEffect, useState } from 'react';
import {
  ActivityIndicator,
  Platform,
  ScrollView,
  StyleSheet,
  Switch,
  View,
} from 'react-native';
import {
  Button,
  Dialog,
  Divider,
  List,
  Portal,
  SegmentedButtons,
  Snackbar,
  Text,
  TextInput,
} from 'react-native-paper';
import { useTranslation } from 'react-i18next';
import { router } from 'expo-router';
import Constants from 'expo-constants';
import { File, Paths } from 'expo-file-system';
import * as Sharing from 'expo-sharing';
import * as WebBrowser from 'expo-web-browser';
import DateTimePicker, {
  type DateTimePickerEvent,
} from '@react-native-community/datetimepicker';

import { useAuthStore } from '@/src/stores/auth';
import * as usersApi from '@/src/api/users';
import { colors, spacing, borderRadius } from '@/src/theme';
import { ROUTES } from '@/src/constants/routes';
import { LEGAL_URLS } from '@/src/constants/legal';
import type { UserProfileResponse } from '@/src/types/auth';

export default function ProfileScreen() {
  const { t, i18n } = useTranslation();
  const logout = useAuthStore((s) => s.logout);
  const updateUser = useAuthStore((s) => s.updateUser);

  // Profile state
  const [profile, setProfile] = useState<UserProfileResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [snackbar, setSnackbar] = useState('');

  // Display name editing
  const [displayName, setDisplayName] = useState('');
  const [savingName, setSavingName] = useState(false);

  // Password dialog
  const [pwdDialogVisible, setPwdDialogVisible] = useState(false);
  const [currentPwd, setCurrentPwd] = useState('');
  const [newPwd, setNewPwd] = useState('');
  const [confirmPwd, setConfirmPwd] = useState('');
  const [pwdError, setPwdError] = useState('');
  const [savingPwd, setSavingPwd] = useState(false);

  // Time picker
  const [showTimePicker, setShowTimePicker] = useState(false);

  // Timezone
  const [autoTimezone, setAutoTimezone] = useState(true);
  const [manualTimezone, setManualTimezone] = useState('');

  // Delete dialog
  const [deleteDialogVisible, setDeleteDialogVisible] = useState(false);
  const [deleting, setDeleting] = useState(false);

  const deviceTimezone = Intl.DateTimeFormat().resolvedOptions().timeZone;

  const loadProfile = useCallback(async () => {
    try {
      setLoading(true);
      const data = await usersApi.getProfile();
      setProfile(data);
      setDisplayName(data.display_name ?? '');
      setAutoTimezone(data.timezone === deviceTimezone);
      setManualTimezone(data.timezone);
    } catch {
      setSnackbar(t('profile.errors.loadFailed'));
    } finally {
      setLoading(false);
    }
  }, [t, deviceTimezone]);

  useEffect(() => {
    loadProfile();
  }, [loadProfile]);

  // ── Save display name ──────────────────────────────────────────────
  async function handleSaveDisplayName() {
    if (!profile) return;
    setSavingName(true);
    try {
      const updated = await usersApi.updateProfile({ display_name: displayName });
      setProfile(updated);
      updateUser({ display_name: updated.display_name });
      setSnackbar(t('profile.account.saved'));
    } catch {
      setSnackbar(t('profile.errors.saveFailed'));
    } finally {
      setSavingName(false);
    }
  }

  // ── Change password ────────────────────────────────────────────────
  async function handleChangePassword() {
    setPwdError('');
    if (newPwd.length < 8) {
      setPwdError(t('auth.validation.passwordMinLength'));
      return;
    }
    if (newPwd !== confirmPwd) {
      setPwdError(t('auth.validation.passwordMismatch'));
      return;
    }
    setSavingPwd(true);
    try {
      await usersApi.changePassword({
        current_password: currentPwd,
        new_password: newPwd,
      });
      setPwdDialogVisible(false);
      resetPasswordFields();
      setSnackbar(t('profile.security.passwordChanged'));
      // Backend invalidates all tokens — force logout after brief delay
      setTimeout(async () => {
        await logout();
        router.replace(ROUTES.LOGIN);
      }, 2000);
    } catch {
      setPwdError(t('profile.errors.passwordChangeFailed'));
    } finally {
      setSavingPwd(false);
    }
  }

  function resetPasswordFields() {
    setCurrentPwd('');
    setNewPwd('');
    setConfirmPwd('');
    setPwdError('');
  }

  // ── Reminder frequency ────────────────────────────────────────────
  async function handleFrequencyChange(freq: string) {
    if (!profile) return;
    try {
      const updated = await usersApi.updateProfile({ reminder_freq: freq });
      setProfile(updated);
    } catch {
      setSnackbar(t('profile.errors.saveFailed'));
    }
  }

  // ── Reminder time ─────────────────────────────────────────────────
  function handleTimeChange(event: DateTimePickerEvent, date?: Date) {
    setShowTimePicker(Platform.OS === 'ios');
    if (event.type === 'set' && date) {
      const hh = String(date.getHours()).padStart(2, '0');
      const mm = String(date.getMinutes()).padStart(2, '0');
      const timeStr = `${hh}:${mm}:00`;
      usersApi
        .updateProfile({ reminder_time: timeStr })
        .then((updated) => setProfile(updated))
        .catch(() => setSnackbar(t('profile.errors.saveFailed')));
    }
  }

  // ── Timezone ──────────────────────────────────────────────────────
  async function handleTimezoneToggle(value: boolean) {
    setAutoTimezone(value);
    if (value) {
      try {
        const updated = await usersApi.updateProfile({ timezone: deviceTimezone });
        setProfile(updated);
        setManualTimezone(deviceTimezone);
      } catch {
        setSnackbar(t('profile.errors.saveFailed'));
      }
    }
  }

  async function handleManualTimezoneSubmit() {
    if (!manualTimezone.trim()) return;
    try {
      const updated = await usersApi.updateProfile({ timezone: manualTimezone.trim() });
      setProfile(updated);
    } catch {
      setSnackbar(t('profile.errors.saveFailed'));
    }
  }

  // ── Language ──────────────────────────────────────────────────────
  async function handleLanguageChange(locale: string) {
    await i18n.changeLanguage(locale);
    try {
      const updated = await usersApi.updateProfile({ locale });
      setProfile(updated);
    } catch {
      setSnackbar(t('profile.errors.saveFailed'));
    }
  }

  // ── Export data ───────────────────────────────────────────────────
  async function handleExport() {
    try {
      const data = await usersApi.exportData();
      const file = new File(Paths.cache, 'offrii-export.json');
      file.create({ overwrite: true });
      file.write(JSON.stringify(data, null, 2));
      await Sharing.shareAsync(file.uri, {
        mimeType: 'application/json',
        dialogTitle: t('profile.data.export'),
      });
    } catch {
      setSnackbar(t('profile.errors.exportFailed'));
    }
  }

  // ── Delete account ────────────────────────────────────────────────
  async function handleDeleteAccount() {
    setDeleting(true);
    try {
      await usersApi.deleteAccount();
      await logout();
      router.replace(ROUTES.LOGIN);
    } catch {
      setSnackbar(t('profile.errors.deleteFailed'));
      setDeleting(false);
    }
  }

  // ── Logout ────────────────────────────────────────────────────────
  async function handleLogout() {
    await logout();
    router.replace(ROUTES.LOGIN);
  }

  // ── Helpers ───────────────────────────────────────────────────────
  function parseTimeToDate(timeStr: string): Date {
    const parts = timeStr.split(':').map(Number);
    const d = new Date();
    d.setHours(parts[0] ?? 0, parts[1] ?? 0, 0, 0);
    return d;
  }

  function formatTime(timeStr: string): string {
    const [h, m] = timeStr.split(':');
    return `${h}:${m}`;
  }

  if (loading) {
    return (
      <View style={styles.centered}>
        <ActivityIndicator size="large" color={colors.primary} />
      </View>
    );
  }

  if (!profile) {
    return (
      <View style={styles.centered}>
        <Text>{t('profile.errors.loadFailed')}</Text>
        <Button mode="outlined" onPress={loadProfile} style={{ marginTop: spacing.md }}>
          {t('common.retry')}
        </Button>
      </View>
    );
  }

  const freqButtons = [
    { value: 'never', label: t('profile.reminders.never') },
    { value: 'daily', label: t('profile.reminders.daily') },
    { value: 'weekly', label: t('profile.reminders.weekly') },
    { value: 'monthly', label: t('profile.reminders.monthly') },
  ];

  const langButtons = [
    { value: 'fr', label: t('profile.language.french') },
    { value: 'en', label: t('profile.language.english') },
  ];

  return (
    <View style={styles.container}>
      <ScrollView contentContainerStyle={styles.scroll}>
        <Text variant="headlineMedium" style={styles.title}>
          {t('profile.title')}
        </Text>

        {/* ── My Account ────────────────────────────────────────── */}
        <List.Section>
          <List.Subheader>{t('profile.account.title')}</List.Subheader>
          <List.Item
            title={t('profile.account.email')}
            description={profile.email}
            left={(props) => <List.Icon {...props} icon="email-outline" />}
          />
          <View style={styles.inlineEdit}>
            <TextInput
              label={t('profile.account.displayName')}
              value={displayName}
              onChangeText={setDisplayName}
              mode="outlined"
              style={styles.flex}
              maxLength={100}
              testID="display-name-input"
            />
            <Button
              mode="contained"
              onPress={handleSaveDisplayName}
              loading={savingName}
              disabled={savingName || displayName === (profile.display_name ?? '')}
              style={styles.saveButton}
              compact
              testID="save-name-button"
            >
              {t('detail.save')}
            </Button>
          </View>
        </List.Section>

        <Divider />

        {/* ── Security ──────────────────────────────────────────── */}
        <List.Section>
          <List.Subheader>{t('profile.security.title')}</List.Subheader>
          <List.Item
            title={t('profile.security.changePassword')}
            left={(props) => <List.Icon {...props} icon="lock-outline" />}
            right={(props) => <List.Icon {...props} icon="chevron-right" />}
            onPress={() => setPwdDialogVisible(true)}
            testID="change-password-button"
          />
        </List.Section>

        <Divider />

        {/* ── Reminders ─────────────────────────────────────────── */}
        <List.Section>
          <List.Subheader>{t('profile.reminders.title')}</List.Subheader>
          <View style={styles.sectionContent}>
            <Text variant="bodyMedium" style={styles.label}>
              {t('profile.reminders.frequency')}
            </Text>
            <SegmentedButtons
              value={profile.reminder_freq}
              onValueChange={handleFrequencyChange}
              buttons={freqButtons}
              density="small"
              style={styles.segmented}
            />
          </View>

          {profile.reminder_freq !== 'never' && (
            <>
              <List.Item
                title={t('profile.reminders.time')}
                description={formatTime(profile.reminder_time)}
                left={(props) => <List.Icon {...props} icon="clock-outline" />}
                onPress={() => setShowTimePicker(true)}
                testID="reminder-time-button"
              />
              {showTimePicker && (
                <DateTimePicker
                  value={parseTimeToDate(profile.reminder_time)}
                  mode="time"
                  is24Hour
                  onChange={handleTimeChange}
                  testID="time-picker"
                />
              )}

              <View style={styles.sectionContent}>
                <View style={styles.switchRow}>
                  <Text variant="bodyMedium">
                    {t('profile.reminders.timezoneAuto')}
                  </Text>
                  <Switch
                    value={autoTimezone}
                    onValueChange={handleTimezoneToggle}
                    trackColor={{ true: colors.primary }}
                    testID="timezone-auto-switch"
                  />
                </View>
                {!autoTimezone && (
                  <TextInput
                    label={t('profile.reminders.timezone')}
                    value={manualTimezone}
                    onChangeText={setManualTimezone}
                    onBlur={handleManualTimezoneSubmit}
                    onSubmitEditing={handleManualTimezoneSubmit}
                    mode="outlined"
                    style={{ marginTop: spacing.sm }}
                    placeholder="Europe/Paris"
                    testID="timezone-input"
                  />
                )}
              </View>
            </>
          )}
        </List.Section>

        <Divider />

        {/* ── Language ──────────────────────────────────────────── */}
        <List.Section>
          <List.Subheader>{t('profile.language.title')}</List.Subheader>
          <View style={styles.sectionContent}>
            <SegmentedButtons
              value={i18n.language}
              onValueChange={handleLanguageChange}
              buttons={langButtons}
              style={styles.segmented}
            />
          </View>
        </List.Section>

        <Divider />

        {/* ── Personal Data ─────────────────────────────────────── */}
        <List.Section>
          <List.Subheader>{t('profile.data.title')}</List.Subheader>
          <List.Item
            title={t('profile.data.export')}
            description={t('profile.data.exportDescription')}
            left={(props) => <List.Icon {...props} icon="download-outline" />}
            onPress={handleExport}
            testID="export-data-button"
          />
          <List.Item
            title={t('profile.data.deleteAccount')}
            titleStyle={{ color: colors.error }}
            left={(props) => <List.Icon {...props} icon="delete-outline" color={colors.error} />}
            onPress={() => setDeleteDialogVisible(true)}
            testID="delete-account-button"
          />
        </List.Section>

        <Divider />

        {/* ── About ─────────────────────────────────────────────── */}
        <List.Section>
          <List.Subheader>{t('profile.about.title')}</List.Subheader>
          <List.Item
            title={t('profile.about.version')}
            description={Constants.expoConfig?.version ?? '–'}
            left={(props) => <List.Icon {...props} icon="information-outline" />}
          />
          <List.Item
            title={t('profile.about.privacy')}
            left={(props) => <List.Icon {...props} icon="shield-check-outline" />}
            right={(props) => <List.Icon {...props} icon="open-in-new" />}
            onPress={() => WebBrowser.openBrowserAsync(LEGAL_URLS.PRIVACY)}
          />
          <List.Item
            title={t('profile.about.terms')}
            left={(props) => <List.Icon {...props} icon="file-document-outline" />}
            right={(props) => <List.Icon {...props} icon="open-in-new" />}
            onPress={() => WebBrowser.openBrowserAsync(LEGAL_URLS.TERMS)}
          />
        </List.Section>

        <Divider />

        {/* ── Logout ────────────────────────────────────────────── */}
        <Button
          mode="outlined"
          onPress={handleLogout}
          textColor={colors.error}
          style={styles.logoutButton}
          testID="logout-button"
        >
          {t('profile.logout')}
        </Button>
      </ScrollView>

      {/* ── Password Dialog ─────────────────────────────────────── */}
      <Portal>
        <Dialog
          visible={pwdDialogVisible}
          onDismiss={() => {
            setPwdDialogVisible(false);
            resetPasswordFields();
          }}
        >
          <Dialog.Title>{t('profile.security.changePassword')}</Dialog.Title>
          <Dialog.Content>
            <TextInput
              label={t('profile.security.currentPassword')}
              value={currentPwd}
              onChangeText={setCurrentPwd}
              secureTextEntry
              mode="outlined"
              style={styles.dialogInput}
              testID="current-password-input"
            />
            <TextInput
              label={t('profile.security.newPassword')}
              value={newPwd}
              onChangeText={setNewPwd}
              secureTextEntry
              mode="outlined"
              style={styles.dialogInput}
              testID="new-password-input"
            />
            <TextInput
              label={t('profile.security.confirmPassword')}
              value={confirmPwd}
              onChangeText={setConfirmPwd}
              secureTextEntry
              mode="outlined"
              style={styles.dialogInput}
              testID="confirm-password-input"
            />
            {pwdError ? (
              <Text variant="bodySmall" style={styles.errorText}>
                {pwdError}
              </Text>
            ) : null}
          </Dialog.Content>
          <Dialog.Actions>
            <Button
              onPress={() => {
                setPwdDialogVisible(false);
                resetPasswordFields();
              }}
            >
              {t('list.deleteConfirm.cancel')}
            </Button>
            <Button
              onPress={handleChangePassword}
              loading={savingPwd}
              disabled={savingPwd || !currentPwd || !newPwd || !confirmPwd}
            >
              {t('detail.save')}
            </Button>
          </Dialog.Actions>
        </Dialog>
      </Portal>

      {/* ── Delete Account Dialog ────────────────────────────────── */}
      <Portal>
        <Dialog
          visible={deleteDialogVisible}
          onDismiss={() => setDeleteDialogVisible(false)}
        >
          <Dialog.Title>{t('profile.data.deleteConfirmTitle')}</Dialog.Title>
          <Dialog.Content>
            <Text variant="bodyMedium">
              {t('profile.data.deleteConfirmMessage')}
            </Text>
          </Dialog.Content>
          <Dialog.Actions>
            <Button onPress={() => setDeleteDialogVisible(false)}>
              {t('list.deleteConfirm.cancel')}
            </Button>
            <Button
              onPress={handleDeleteAccount}
              loading={deleting}
              disabled={deleting}
              textColor={colors.error}
            >
              {t('profile.data.deleteConfirmButton')}
            </Button>
          </Dialog.Actions>
        </Dialog>
      </Portal>

      {/* ── Snackbar ─────────────────────────────────────────────── */}
      <Snackbar
        visible={!!snackbar}
        onDismiss={() => setSnackbar('')}
        duration={3000}
      >
        {snackbar}
      </Snackbar>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: colors.background,
  },
  centered: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: spacing.lg,
  },
  scroll: {
    paddingTop: spacing.xl + spacing.lg,
    paddingBottom: spacing.xl * 2,
  },
  title: {
    paddingHorizontal: spacing.lg,
    marginBottom: spacing.sm,
  },
  sectionContent: {
    paddingHorizontal: spacing.lg,
    paddingVertical: spacing.sm,
  },
  label: {
    marginBottom: spacing.xs,
    color: colors.textSecondary,
  },
  segmented: {
    marginTop: spacing.xs,
  },
  inlineEdit: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: spacing.lg,
    gap: spacing.sm,
  },
  flex: {
    flex: 1,
  },
  saveButton: {
    borderRadius: borderRadius.sm,
  },
  switchRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  logoutButton: {
    marginHorizontal: spacing.lg,
    marginTop: spacing.xl,
    borderColor: colors.error,
    borderRadius: borderRadius.sm,
  },
  dialogInput: {
    marginBottom: spacing.sm,
  },
  errorText: {
    color: colors.error,
    marginTop: spacing.xs,
  },
});
