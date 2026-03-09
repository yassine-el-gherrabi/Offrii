import SwiftUI

struct ReminderSettingsView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var frequency = "never"
    @State private var time = Calendar.current.date(from: DateComponents(hour: 9, minute: 0)) ?? Date()
    @State private var isSaving = false
    @State private var hasLoaded = false

    private let frequencies = ["never", "daily", "weekly", "monthly"]

    var body: some View {
        ZStack {
            OffriiTheme.cardSurface.ignoresSafeArea()

            ScrollView {
                VStack(spacing: OffriiTheme.spacingMD) {
                    OffriiCard {
                        VStack(spacing: OffriiTheme.spacingMD) {
                            // Frequency picker
                            VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                                Text(NSLocalizedString("profile.reminderFrequency", comment: ""))
                                    .font(OffriiTypography.headline)
                                    .foregroundColor(OffriiTheme.text)

                                Picker("", selection: $frequency) {
                                    ForEach(frequencies, id: \.self) { freq in
                                        Text(frequencyLabel(freq)).tag(freq)
                                    }
                                }
                                .pickerStyle(.segmented)
                            }

                            if frequency != "never" {
                                // Time picker
                                VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                                    Text(NSLocalizedString("profile.reminderTime", comment: ""))
                                        .font(OffriiTypography.headline)
                                        .foregroundColor(OffriiTheme.text)

                                    DatePicker(
                                        "",
                                        selection: $time,
                                        displayedComponents: .hourAndMinute
                                    )
                                    .datePickerStyle(.wheel)
                                    .labelsHidden()
                                }
                            }

                            OffriiButton(
                                NSLocalizedString("common.save", comment: ""),
                                isLoading: isSaving
                            ) {
                                Task { await save() }
                            }
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                }
                .padding(.top, OffriiTheme.spacingMD)
            }
        }
        .navigationTitle(NSLocalizedString("profile.reminders", comment: ""))
        .navigationBarTitleDisplayMode(.inline)
        .task {
            guard !hasLoaded else { return }
            hasLoaded = true
            await loadSettings()
        }
    }

    private func frequencyLabel(_ freq: String) -> String {
        switch freq {
        case "daily": return NSLocalizedString("reminder.daily", comment: "")
        case "weekly": return NSLocalizedString("reminder.weekly", comment: "")
        case "monthly": return NSLocalizedString("reminder.monthly", comment: "")
        default: return NSLocalizedString("reminder.never", comment: "")
        }
    }

    private func loadSettings() async {
        do {
            let profile = try await UserService.shared.getProfile()
            frequency = profile.reminderFreq ?? "never"
            // Parse time string "HH:mm" into Date
            let timeString = profile.reminderTime ?? "09:00"
            let parts = timeString.split(separator: ":")
            if parts.count == 2,
               let hour = Int(parts[0]),
               let minute = Int(parts[1]),
               let date = Calendar.current.date(from: DateComponents(hour: hour, minute: minute)) {
                time = date
            }
        } catch {
            // Keep defaults
        }
    }

    private func save() async {
        isSaving = true
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm"
        let timeString = formatter.string(from: time)

        do {
            _ = try await UserService.shared.updateProfile(
                displayName: nil,
                reminderFreq: frequency,
                reminderTime: timeString,
                timezone: TimeZone.current.identifier,
                locale: nil
            )
        } catch {
            // Silent failure — could show alert
        }
        isSaving = false
    }
}
