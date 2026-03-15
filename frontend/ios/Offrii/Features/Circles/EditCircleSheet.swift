import SwiftUI

struct EditCircleSheet: View {
    @Environment(\.dismiss) private var dismiss
    let circleId: UUID
    let currentName: String
    let onSaved: () -> Void

    @State private var name: String
    @State private var isSaving = false
    @State private var error: String?

    init(circleId: UUID, currentName: String, onSaved: @escaping () -> Void) {
        self.circleId = circleId
        self.currentName = currentName
        self.onSaved = onSaved
        _name = State(initialValue: currentName)
    }

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.background.ignoresSafeArea()

                VStack(spacing: OffriiTheme.spacingLG) {
                    OffriiTextField(
                        label: NSLocalizedString("circles.createName", comment: ""),
                        text: $name,
                        placeholder: NSLocalizedString("circles.createNamePlaceholder", comment: ""),
                        errorMessage: error
                    )

                    OffriiButton(
                        NSLocalizedString("circles.edit.save", comment: ""),
                        isLoading: isSaving,
                        isDisabled: name.trimmingCharacters(in: .whitespaces).isEmpty
                    ) {
                        Task { await save() }
                    }

                    Spacer()
                }
                .padding(OffriiTheme.spacingLG)
            }
            .navigationTitle(NSLocalizedString("circles.edit.title", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) {
                        dismiss()
                    }
                    .foregroundColor(OffriiTheme.primary)
                }
            }
        }
    }

    private func save() async {
        let trimmed = name.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }

        isSaving = true
        error = nil
        do {
            _ = try await CircleService.shared.updateCircle(id: circleId, name: trimmed)
            onSaved()
            dismiss()
        } catch {
            self.error = error.localizedDescription
        }
        isSaving = false
    }
}
