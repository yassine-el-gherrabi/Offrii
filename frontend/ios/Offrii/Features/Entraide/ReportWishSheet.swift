import SwiftUI

// MARK: - ReportWishSheet

struct ReportWishSheet: View {
    let wishId: UUID
    @State private var selectedReason: WishReportReason?
    @State private var isSubmitting = false
    @State private var isSuccess = false
    @State private var error: String?
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            VStack(spacing: OffriiTheme.spacingLG) {
                Text("entraide.report.subtitle")
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textSecondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                // Reason buttons
                VStack(spacing: OffriiTheme.spacingSM) {
                    ForEach(WishReportReason.allCases) { reason in
                        Button {
                            selectedReason = reason
                        } label: {
                            HStack {
                                Text(reason.label)
                                    .font(OffriiTypography.body)
                                    .foregroundColor(OffriiTheme.text)
                                Spacer()
                                if selectedReason == reason {
                                    Image(systemName: "checkmark.circle.fill")
                                        .foregroundColor(OffriiTheme.primary)
                                }
                            }
                            .padding(OffriiTheme.spacingBase)
                            .background(
                                selectedReason == reason
                                    ? OffriiTheme.primary.opacity(0.08)
                                    : OffriiTheme.card
                            )
                            .cornerRadius(OffriiTheme.cornerRadiusMD)
                            .overlay(
                                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                                    .strokeBorder(
                                        selectedReason == reason
                                            ? OffriiTheme.primary
                                            : OffriiTheme.border,
                                        lineWidth: 1
                                    )
                            )
                        }
                        .buttonStyle(.plain)
                    }
                }

                if let error {
                    Text(error)
                        .font(OffriiTypography.footnote)
                        .foregroundColor(OffriiTheme.danger)
                }

                Spacer()

                // Submit
                OffriiButton(
                    NSLocalizedString("entraide.report.submit", comment: ""),
                    variant: .danger,
                    isLoading: isSubmitting,
                    isDisabled: selectedReason == nil
                ) {
                    Task { await submitReport() }
                }
            }
            .padding(OffriiTheme.spacingLG)
            .background(OffriiTheme.background)
            .navigationTitle(NSLocalizedString("entraide.report.title", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) {
                        dismiss()
                    }
                }
            }
        }
    }

    private func submitReport() async {
        guard let reason = selectedReason else { return }
        isSubmitting = true
        error = nil

        do {
            try await CommunityWishService.shared.reportWish(id: wishId, reason: reason)
            isSubmitting = false
            dismiss()
        } catch {
            self.error = error.localizedDescription
            isSubmitting = false
        }
    }
}
