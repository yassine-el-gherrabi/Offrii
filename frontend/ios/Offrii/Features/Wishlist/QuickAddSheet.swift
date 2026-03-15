import SwiftUI

struct QuickAddSheet: View {
    @Environment(\.dismiss) private var dismiss
    @State private var name = ""
    @State private var priceText = ""
    @State private var selectedCategoryId: UUID?
    @State private var priority: Int = 2
    @State private var linkText = ""
    @State private var selectedImage: UIImage?
    @State private var isAdding = false
    @State private var categories: [CategoryResponse] = []

    let onAdd: (String, Decimal?, UUID?, Int, String?, [String]?) async -> Bool

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: OffriiTheme.spacingBase) {
                    // Name (required)
                    OffriiTextField(
                        label: NSLocalizedString("wishlist.quickAdd.placeholder", comment: ""),
                        text: $name,
                        placeholder: NSLocalizedString("wishlist.quickAdd.placeholder", comment: ""),
                        textContentType: nil,
                        autocapitalization: .sentences
                    )

                    // Price (optional)
                    OffriiTextField(
                        label: NSLocalizedString("item.estimatedPrice", comment: ""),
                        text: $priceText,
                        placeholder: "0,00 €",
                        keyboardType: .decimalPad
                    )

                    // Category chips (optional)
                    if !categories.isEmpty {
                        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                            Text(NSLocalizedString("item.category", comment: ""))
                                .font(OffriiTypography.subheadline)
                                .foregroundColor(OffriiTheme.textMuted)

                            ScrollView(.horizontal, showsIndicators: false) {
                                HStack(spacing: OffriiTheme.spacingSM) {
                                    ForEach(categories, id: \.id) { cat in
                                        OffriiChip(
                                            title: cat.name,
                                            isSelected: selectedCategoryId == cat.id
                                        ) {
                                            selectedCategoryId = selectedCategoryId == cat.id ? nil : cat.id
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Priority (optional)
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                        Text(NSLocalizedString("item.priority", comment: ""))
                            .font(OffriiTypography.subheadline)
                            .foregroundColor(OffriiTheme.textMuted)

                        HStack(spacing: OffriiTheme.spacingSM) {
                            priorityButton(level: 1, label: NSLocalizedString("priority.low", comment: ""), color: OffriiTheme.textMuted)
                            priorityButton(level: 2, label: NSLocalizedString("priority.medium", comment: ""), color: OffriiTheme.accent)
                            priorityButton(level: 3, label: NSLocalizedString("priority.high", comment: ""), color: OffriiTheme.danger)
                        }
                    }

                    // Photo (optional)
                    OffriiImagePicker(selectedImage: $selectedImage)

                    // Link (optional, single for quick add)
                    OffriiTextField(
                        label: NSLocalizedString("item.url", comment: ""),
                        text: $linkText,
                        placeholder: "https://...",
                        keyboardType: .URL,
                        autocapitalization: .never
                    )

                    // Submit
                    OffriiButton(
                        NSLocalizedString("wishlist.quickAdd.button", comment: ""),
                        isLoading: isAdding,
                        isDisabled: name.trimmingCharacters(in: .whitespaces).isEmpty
                    ) {
                        Task {
                            isAdding = true

                            // Upload image if selected
                            var imageUrl: String?
                            if let image = selectedImage,
                               let data = image.compressForUpload() {
                                imageUrl = try? await ItemService.shared.uploadImage(data)
                            }

                            let price = Decimal(string: priceText.replacingOccurrences(of: ",", with: "."))
                            let trimmedLink = linkText.trimmingCharacters(in: .whitespaces)
                            let links: [String]? = trimmedLink.isEmpty ? nil : [trimmedLink]

                            let success = await onAdd(
                                name.trimmingCharacters(in: .whitespaces),
                                price,
                                selectedCategoryId,
                                priority,
                                imageUrl,
                                links
                            )
                            isAdding = false
                            if success { dismiss() }
                        }
                    }

                    Spacer()
                }
                .padding(OffriiTheme.spacingLG)
            }
            .background(OffriiTheme.background.ignoresSafeArea())
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
            .task {
                do {
                    categories = try await CategoryService.shared.listCategories()
                } catch {}
            }
        }
        .presentationDetents([.medium, .large])
    }

    private func priorityButton(level: Int, label: String, color: Color) -> some View {
        let isSelected = priority == level
        return Button {
            OffriiHaptics.selection()
            priority = level
        } label: {
            HStack(spacing: 4) {
                if level >= 2 {
                    Circle()
                        .fill(color)
                        .frame(width: 8, height: 8)
                }
                Text(label)
                    .font(OffriiTypography.footnote)
                    .fontWeight(isSelected ? .semibold : .regular)
            }
            .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
            .padding(.horizontal, OffriiTheme.spacingMD)
            .padding(.vertical, OffriiTheme.spacingSM)
            .background(isSelected ? color : OffriiTheme.surface)
            .cornerRadius(OffriiTheme.cornerRadiusSM)
        }
        .buttonStyle(.plain)
        .animation(OffriiAnimation.snappy, value: isSelected)
    }
}
