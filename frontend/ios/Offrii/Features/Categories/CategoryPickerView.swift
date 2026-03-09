import SwiftUI

struct CategoryPickerView: View {
    let categories: [CategoryResponse]
    @Binding var selectedId: UUID?
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            List {
                // "None" option
                Button {
                    selectedId = nil
                    dismiss()
                } label: {
                    HStack {
                        Text(NSLocalizedString("wishlist.allCategories", comment: ""))
                            .font(OffriiTypography.body)
                            .foregroundColor(OffriiTheme.text)
                        Spacer()
                        if selectedId == nil {
                            Image(systemName: "checkmark")
                                .foregroundColor(OffriiTheme.primary)
                        }
                    }
                }

                ForEach(categories, id: \.id) { category in
                    Button {
                        selectedId = category.id
                        dismiss()
                    } label: {
                        HStack {
                            if let icon = category.icon {
                                Text(icon)
                            }
                            Text(category.name)
                                .font(OffriiTypography.body)
                                .foregroundColor(OffriiTheme.text)
                            Spacer()
                            if selectedId == category.id {
                                Image(systemName: "checkmark")
                                    .foregroundColor(OffriiTheme.primary)
                            }
                        }
                    }
                }
            }
            .listStyle(.insetGrouped)
            .navigationTitle(NSLocalizedString("item.category", comment: ""))
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
}
