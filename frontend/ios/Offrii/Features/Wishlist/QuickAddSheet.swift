import SwiftUI

struct QuickAddSheet: View {
    @Environment(\.dismiss) private var dismiss
    @State private var name = ""
    @State private var isAdding = false
    let onAdd: (String) async -> Bool

    var body: some View {
        NavigationStack {
            VStack(spacing: OffriiTheme.spacingMD) {
                OffriiTextField(
                    label: NSLocalizedString("wishlist.quickAdd.placeholder", comment: ""),
                    text: $name,
                    placeholder: NSLocalizedString("wishlist.quickAdd.placeholder", comment: ""),
                    textContentType: nil,
                    autocapitalization: .sentences
                )

                OffriiButton(
                    NSLocalizedString("wishlist.quickAdd.button", comment: ""),
                    isLoading: isAdding,
                    isDisabled: name.trimmingCharacters(in: .whitespaces).isEmpty
                ) {
                    Task {
                        isAdding = true
                        let success = await onAdd(name.trimmingCharacters(in: .whitespaces))
                        isAdding = false
                        if success { dismiss() }
                    }
                }

                Spacer()
            }
            .padding(OffriiTheme.spacingLG)
            .background(OffriiTheme.cardSurface.ignoresSafeArea())
            .navigationTitle(NSLocalizedString("wishlist.quickAdd.title", comment: ""))
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
        .presentationDetents([.medium])
    }
}
