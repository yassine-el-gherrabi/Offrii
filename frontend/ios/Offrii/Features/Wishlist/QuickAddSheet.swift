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
                            priorityButton(level: 1, label: NSLocalizedString("priority.low", comment: ""), dots: 1, opacity: 0.4)
                            priorityButton(level: 2, label: NSLocalizedString("priority.medium", comment: ""), dots: 2, opacity: 0.7)
                            priorityButton(level: 3, label: NSLocalizedString("priority.high", comment: ""), dots: 3, opacity: 1.0)
                        }
                    }

                    // Photo (optional)
                    OffriiImagePicker(selectedImage: $selectedImage, isUploading: isAdding)

                    // Link (optional, single for quick add)
                    OffriiTextField(
                        label: NSLocalizedString("item.url", comment: ""),
                        text: $linkText,
                        placeholder: "https://...",
                        keyboardType: .URL,
                        autocapitalization: .never
                    )

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
                } catch {}
            }
        }
        .presentationDetents([.medium, .large])
    }

    private func priorityButton(level: Int, label: String, dots: Int, opacity: Double) -> some View {
        let isSelected = priority == level
        let color = OffriiTheme.primary.opacity(opacity)
        return Button {
            OffriiHaptics.selection()
            priority = level
        } label: {
            HStack(spacing: 3) {
                HStack(spacing: 2) {
                    ForEach(0..<dots, id: \.self) { _ in
                        Circle()
                            .fill(isSelected ? .white : color)
                            .frame(width: 6, height: 6)
                    }
                }
                Text(label)
                    .font(OffriiTypography.footnote)
                    .fontWeight(isSelected ? .semibold : .regular)
            }
            .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
            .padding(.horizontal, OffriiTheme.spacingMD)
            .padding(.vertical, OffriiTheme.spacingSM)
            .background(isSelected ? OffriiTheme.primary.opacity(opacity) : OffriiTheme.surface)
            .cornerRadius(OffriiTheme.cornerRadiusSM)
        }
        .buttonStyle(.plain)
        .animation(OffriiAnimation.snappy, value: isSelected)
    }
}
