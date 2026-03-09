import SwiftUI

struct CreateCircleSheet: View {
    @Environment(\.dismiss) private var dismiss
    @State private var circleName = ""
    @State private var isCreating = false
    @State private var error: String?
    let onCreated: (OffriiCircle) -> Void

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.cardSurface.ignoresSafeArea()

                VStack(spacing: OffriiTheme.spacingLG) {
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                        Text(NSLocalizedString("circles.createName", comment: ""))
                            .font(OffriiTypography.headline)
                            .foregroundColor(OffriiTheme.text)

                        TextField(
                            NSLocalizedString("circles.createNamePlaceholder", comment: ""),
                            text: $circleName
                        )
                        .font(OffriiTypography.body)
                        .padding(OffriiTheme.spacingMD)
                        .background(OffriiTheme.card)
                        .cornerRadius(OffriiTheme.cornerRadiusSM)
                        .overlay(
                            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusSM)
                                .stroke(OffriiTheme.border, lineWidth: 1)
                        )
                    }

                    if let error {
                        Text(error)
                            .font(OffriiTypography.caption)
                            .foregroundColor(OffriiTheme.danger)
                    }

                    OffriiButton(
                        NSLocalizedString("circles.createButton", comment: ""),
                        isLoading: isCreating
                    ) {
                        Task { await createCircle() }
                    }
                    .disabled(circleName.trimmingCharacters(in: .whitespaces).isEmpty)

                    Spacer()
                }
                .padding(OffriiTheme.spacingLG)
            }
            .navigationTitle(NSLocalizedString("circles.create", comment: ""))
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

    private func createCircle() async {
        let name = circleName.trimmingCharacters(in: .whitespaces)
        guard !name.isEmpty else { return }

        isCreating = true
        error = nil
        do {
            let circle = try await CircleService.shared.createCircle(name: name)
            onCreated(circle)
            dismiss()
        } catch {
            self.error = error.localizedDescription
        }
        isCreating = false
    }
}
