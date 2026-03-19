import SwiftUI

// MARK: - Report Wish Sheet

struct ReportWishSheet: View {
    let wishId: UUID

    @Environment(\.dismiss) private var dismiss
    @State private var selectedReason: WishReportReason?
    @State private var isSubmitting = false
    @State private var error: String?
    @State private var showSuccess = false
    @State private var otherDetails = ""

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

                if selectedReason == .other {
                    TextField(
                        NSLocalizedString("entraide.report.otherPlaceholder", comment: ""),
                        text: $otherDetails,
                        axis: .vertical
                    )
                    .font(OffriiTypography.body)
                    .lineLimit(2...4)
                    .padding(OffriiTheme.spacingSM)
                    .background(OffriiTheme.surface)
                    .cornerRadius(OffriiTheme.cornerRadiusMD)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                            .stroke(OffriiTheme.border, lineWidth: 1)
                    )
                }

                if let error {
                    Text(error)
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.danger)
                }

                if showSuccess {
                    Label(
                        NSLocalizedString("entraide.report.success", comment: ""),
                        systemImage: "checkmark.circle.fill"
                    )
                    .font(OffriiTypography.footnote)
                    .foregroundColor(OffriiTheme.success)
                    .padding(OffriiTheme.spacingSM)
                    .frame(maxWidth: .infinity)
                    .background(OffriiTheme.success.opacity(0.1))
                    .cornerRadius(OffriiTheme.cornerRadiusMD)
                } else {
                    OffriiButton(
                        NSLocalizedString("entraide.report.submit", comment: ""),
                        variant: .danger,
                        isLoading: isSubmitting,
                        isDisabled: selectedReason == nil
                    ) {
                        Task { await submit() }
                    }
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
            showSuccess = true
            try? await Task.sleep(for: .seconds(2))
            dismiss()
        } catch {
            self.error = error.localizedDescription
        }
        isSubmitting = false
    }
}
