import SwiftUI

struct InviteContactsSheet: View {
    @Environment(\.dismiss) private var dismiss
    @State private var selectedNames: [String] = []
    @State private var showPicker = true

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.background.ignoresSafeArea()

                if selectedNames.isEmpty {
                    VStack(spacing: OffriiTheme.spacingLG) {
                        Spacer()

                        Image(systemName: "person.crop.circle.badge.plus")
                            .font(.system(size: 48))
                            .foregroundColor(OffriiTheme.primary)

                        Text(NSLocalizedString("friends.inviteContacts", comment: ""))
                            .font(OffriiTypography.headline)
                            .foregroundColor(OffriiTheme.text)
                            .multilineTextAlignment(.center)

                        OffriiButton(
                            NSLocalizedString("friends.inviteContacts", comment: "")
                        ) {
                            showPicker = true
                        }
                        .padding(.horizontal, OffriiTheme.spacingXL)

                        Spacer()
                    }
                } else {
                    VStack(spacing: OffriiTheme.spacingLG) {
                        List {
                            ForEach(selectedNames, id: \.self) { name in
                                HStack {
                                    Text(name)
                                        .font(OffriiTypography.body)
                                        .foregroundColor(OffriiTheme.text)

                                    Spacer()

                                    Image(systemName: "checkmark.circle.fill")
                                        .foregroundColor(OffriiTheme.success)
                                }
                            }
                        }
                        .listStyle(.plain)

                        OffriiButton(
                            NSLocalizedString("friends.shareInviteLink", comment: "")
                        ) {
                            shareWithContacts()
                        }
                        .padding(.horizontal, OffriiTheme.spacingLG)
                        .padding(.bottom, OffriiTheme.spacingLG)
                    }
                }
            }
            .navigationTitle(NSLocalizedString("friends.inviteContacts", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) {
                        dismiss()
                    }
                    .foregroundColor(OffriiTheme.primary)
                }
            }
            .sheet(isPresented: $showPicker) {
                ContactPickerRepresentable(
                    onSelect: { names in
                        selectedNames = names
                        showPicker = false
                    },
                    onCancel: {
                        showPicker = false
                    }
                )
            }
        }
    }

    private func shareWithContacts() {
        let url = URL(string: "https://offrii.com/invite")!
        let message = "Rejoins-moi sur Offrii pour partager nos envies ! \(url)"
        let activityVC = UIActivityViewController(
            activityItems: [message],
            applicationActivities: nil
        )
        if let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
           let rootVC = windowScene.windows.first?.rootViewController {
            rootVC.present(activityVC, animated: true)
        }
        dismiss()
    }
}
