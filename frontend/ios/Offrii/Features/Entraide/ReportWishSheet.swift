import SwiftUI

// MARK: - Report Wish Sheet

struct ReportWishSheet: View {
    let wishId: UUID

    @Environment(\.dismiss) private var dismiss
    @State private var selectedReason: WishReportReason?
    @State private var isSubmitting = false
    @State private var error: String?

    var body: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {
                Text(NSLocalizedString("entraide.report.subtitle", comment: ""))
                    .font(OffriiTypography.body)
                    .foregroundColor(OffriiTheme.textSecondary)

                VStack(spacing: OffriiTheme.spacingSM) {
                    ForEach(WishReportReason.allCases) { reason in
                        let isSelected = selectedReason == reason
                        Button {
                            withAnimation(OffriiAnimation.snappy) {
                                selectedReason = reason
                            }
                            OffriiHaptics.selection()
                        } label: {
                            HStack {
                                Text(reason.label)
                                    .font(OffriiTypography.body)
                                    .foregroundColor(OffriiTheme.text)
                                Spacer()
                                Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                                    .foregroundColor(isSelected ? OffriiTheme.primary : OffriiTheme.textMuted)
                            }
                            .padding(OffriiTheme.spacingBase)
                            .background(isSelected ? OffriiTheme.primary.opacity(0.05) : OffriiTheme.card)
                            .cornerRadius(OffriiTheme.cornerRadiusMD)
                            .overlay(
                                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                                    .strokeBorder(
                                        isSelected ? OffriiTheme.primary : OffriiTheme.border,
                                        lineWidth: 1
                                    )
                            )
                        }
                        .buttonStyle(.plain)
                    }
                }

                if let error {
                    Text(error)
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.danger)
                }

                OffriiButton(
                    NSLocalizedString("entraide.report.submit", comment: ""),
                    variant: .danger,
                    isLoading: isSubmitting,
                    isDisabled: selectedReason == nil
                ) {
                    Task { await submit() }
                }

                Spacer()
            }
            .padding(OffriiTheme.spacingLG)
            .background(OffriiTheme.background)
            .navigationTitle(NSLocalizedString("entraide.report.title", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) { dismiss() }
                }
            }
        }
    }

    private func submit() async {
        guard let reason = selectedReason else { return }
        isSubmitting = true
        error = nil

        do {
            try await CommunityWishService.shared.reportWish(id: wishId, reason: reason)
            OffriiHaptics.success()
            dismiss()
        } catch {
            self.error = error.localizedDescription
        }
        isSubmitting = false
    }
}
