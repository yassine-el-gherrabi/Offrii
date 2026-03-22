import SwiftUI

struct QuickAddSheet: View {
    @Environment(\.dismiss) private var dismiss
    @State private var name = ""
    @State private var priceText = ""
    @State private var selectedCategoryId: UUID?
    @State private var priority: Int = 2
    @State private var linkText = ""
    @State private var selectedImage: UIImage?
    @State private var isPrivate = false
    @State private var isAdding = false
    @State private var categories: [CategoryResponse] = []
    @State private var linkValidationError: String?

    let onAdd: (String, Decimal?, UUID?, Int, String?, [String]?, Bool) async -> Bool

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
                            priorityButton(level: 1, label: NSLocalizedString("priority.low", comment: ""), flames: 1)
                            priorityButton(level: 2, label: NSLocalizedString("priority.medium", comment: ""), flames: 2)
                            priorityButton(level: 3, label: NSLocalizedString("priority.high", comment: ""), flames: 3)
                        }
                    }

                    // Photo (optional)
                    OffriiImagePicker(selectedImage: $selectedImage, isUploading: isAdding)

                    // Link (optional, single for quick add)
                    VStack(alignment: .leading, spacing: 2) {
                        OffriiTextField(
                            label: NSLocalizedString("item.url", comment: ""),
                            text: $linkText,
                            placeholder: "google.com, amazon.fr...",
                            keyboardType: .URL,
                            autocapitalization: .never
                        )

                        if let linkValidationError {
                            Text(linkValidationError)
                                .font(OffriiTypography.caption)
                                .foregroundColor(OffriiTheme.danger)
                        }
                    }

                    // Private toggle
                    Toggle(isOn: $isPrivate) {
                        HStack {
                            Image(systemName: "lock.fill")
                                .foregroundColor(OffriiTheme.textMuted)
                            VStack(alignment: .leading, spacing: 2) {
                                Text(NSLocalizedString("wishlist.private", comment: ""))
                                    .font(OffriiTypography.body)
                                Text(NSLocalizedString("wishlist.privateHint", comment: ""))
                                    .font(OffriiTypography.caption)
                                    .foregroundColor(OffriiTheme.textMuted)
                            }
                        }
                    }
                    .tint(OffriiTheme.primary)

                    // Submit
                    OffriiButton(
                        NSLocalizedString("wishlist.quickAdd.button", comment: ""),
                        isLoading: isAdding,
                        isDisabled: name.trimmingCharacters(in: .whitespaces).isEmpty
                    ) {
                        Task {
                            linkValidationError = nil

                            let normalizedLink = normalizeURL(linkText)
                            if !normalizedLink.isEmpty && !isValidURL(normalizedLink) {
                                linkValidationError = NSLocalizedString("error.invalidLink", comment: "")
                                return
                            }

                            isAdding = true

                            // Upload image if selected
                            var imageUrl: String?
                            if let image = selectedImage,
                               let data = image.compressForUpload() {
                                imageUrl = try? await ItemService.shared.uploadImage(data)
                            }

                            let price = Decimal(string: priceText.replacingOccurrences(of: ",", with: "."))
                            let links: [String]? = normalizedLink.isEmpty ? nil : [normalizedLink]

                            let success = await onAdd(
                                name.trimmingCharacters(in: .whitespaces),
                                price,
                                selectedCategoryId,
                                priority,
                                imageUrl,
                                links,
                                isPrivate
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
                } catch { /* Best-effort refresh */ }
            }
        }
        .presentationDetents([.medium, .large])
    }

    private func priorityButton(level: Int, label: String, flames: Int) -> some View {
        let isSelected = priority == level
        return Button {
            OffriiHaptics.selection()
            priority = level
        } label: {
            HStack(spacing: 2) {
                HStack(spacing: -2) {
                    ForEach(0..<flames, id: \.self) { _ in
                        Image(systemName: "flame.fill")
                            .font(.system(size: 10))
                    }
                }
                Text(label)
                    .font(OffriiTypography.footnote)
                    .fontWeight(isSelected ? .semibold : .regular)
            }
            .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
            .padding(.horizontal, OffriiTheme.spacingMD)
            .padding(.vertical, OffriiTheme.spacingSM)
            .background(isSelected ? OffriiTheme.primary : OffriiTheme.surface)
            .cornerRadius(OffriiTheme.cornerRadiusSM)
        }
        .buttonStyle(.plain)
        .animation(OffriiAnimation.snappy, value: isSelected)
    }
}
