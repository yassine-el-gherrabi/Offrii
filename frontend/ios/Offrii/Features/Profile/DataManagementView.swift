import SwiftUI

struct DataManagementView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var isExporting = false
    @State private var showDeleteAlert = false
    @State private var deleteConfirmation = ""
    @State private var showDeleteConfirm = false
    @State private var isDeleting = false
    @State private var exportFileURL: URL?
    @State private var showShareSheet = false

    var body: some View {
        ZStack {
            OffriiTheme.surface.ignoresSafeArea()

            ScrollView {
                VStack(spacing: OffriiTheme.spacingBase) {
                    // Export section
                    OffriiCard {
                        VStack(spacing: OffriiTheme.spacingBase) {
                            HStack {
                                Image(systemName: "square.and.arrow.up")
                                    .foregroundColor(OffriiTheme.primary)
                                Text(NSLocalizedString("profile.exportData", comment: ""))
                                    .font(OffriiTypography.headline)
                                    .foregroundColor(OffriiTheme.text)
                                Spacer()
                            }

                            OffriiButton(
                                NSLocalizedString("profile.exportData", comment: ""),
                                variant: .secondary,
                                isLoading: isExporting
                            ) {
                                Task { await exportUserData() }
                            }
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)

                    // Delete section
                    OffriiCard {
                        VStack(spacing: OffriiTheme.spacingBase) {
                            HStack {
                                Image(systemName: "trash.fill")
                                    .foregroundColor(OffriiTheme.danger)
                                Text(NSLocalizedString("profile.deleteAccount", comment: ""))
                                    .font(OffriiTypography.headline)
                                    .foregroundColor(OffriiTheme.danger)
                                Spacer()
                            }

                            Text(NSLocalizedString("profile.deleteAccount.message", comment: ""))
                                .font(OffriiTypography.body)
                                .foregroundColor(OffriiTheme.textSecondary)
                                .frame(maxWidth: .infinity, alignment: .leading)

                            OffriiButton(
                                NSLocalizedString("profile.deleteAccount", comment: ""),
                                variant: .danger
                            ) {
                                showDeleteAlert = true
                            }
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                }
                .padding(.top, OffriiTheme.spacingBase)
            }
        }
        .navigationTitle(NSLocalizedString("profile.data", comment: ""))
        .navigationBarTitleDisplayMode(.inline)
        .alert(
            NSLocalizedString("profile.deleteAccount.title", comment: ""),
            isPresented: $showDeleteAlert
        ) {
            TextField(
                NSLocalizedString("profile.deleteAccount.placeholder", comment: ""),
                text: $deleteConfirmation
            )
            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {
                deleteConfirmation = ""
            }
            Button(NSLocalizedString("common.delete", comment: ""), role: .destructive) {
                guard isDeleteConfirmed else { return }
                Task { await deleteAccount() }
            }
            .disabled(!isDeleteConfirmed)
        } message: {
            Text(NSLocalizedString("profile.deleteAccount.confirm", comment: ""))
        }
        .sheet(isPresented: $showShareSheet) {
            if let url = exportFileURL {
                ShareSheet(items: [url])
            }
        }
    }

    private var isDeleteConfirmed: Bool {
        let target = NSLocalizedString("profile.deleteAccount.placeholder", comment: "")
        return deleteConfirmation.uppercased() == target.uppercased()
    }

    private func exportUserData() async {
        isExporting = true
        do {
            let data = try await UserService.shared.exportDataRaw()
            let tempDir = FileManager.default.temporaryDirectory
            let fileURL = tempDir.appendingPathComponent("offrii-export.json")
            try data.write(to: fileURL)
            exportFileURL = fileURL
            showShareSheet = true
        } catch { /* Best-effort */ }
        isExporting = false
    }

    private func deleteAccount() async {
        isDeleting = true
        do {
            try await UserService.shared.deleteAccount()
            await authManager.logout()
        } catch {
            // Could show error alert
        }
        isDeleting = false
        deleteConfirmation = ""
    }
}

// MARK: - Share Sheet

struct ShareSheet: UIViewControllerRepresentable {
    let items: [Any]

    func makeUIViewController(context: Context) -> UIActivityViewController {
        UIActivityViewController(activityItems: items, applicationActivities: nil)
    }

    func updateUIViewController(_ uiViewController: UIActivityViewController, context: Context) {}
}
