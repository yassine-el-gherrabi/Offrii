import SwiftUI

struct ItemEditView: View {
    let item: Item
    let onSave: (Item) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var name: String
    @State private var description: String
    @State private var links: [String]
    @State private var estimatedPrice: String
    @State private var priority: Int
    @State private var categoryId: UUID?
    @State private var isSaving = false
    @State private var categories: [CategoryResponse] = []
    @State private var showCategoryPicker = false
    @State private var selectedImage: UIImage?
    @State private var isUploadingImage = false
    @State private var isPrivate: Bool
    @State private var showShareToCircle = false
    @State private var uploadError: String?

    init(item: Item, onSave: @escaping (Item) -> Void) {
        self.item = item
        self.onSave = onSave
        _name = State(initialValue: item.name)
        _description = State(initialValue: item.description ?? "")
        _links = State(initialValue: item.links ?? (item.url.map { [$0] } ?? [""]))
        _estimatedPrice = State(initialValue: item.estimatedPrice.map { "\($0)" } ?? "")
        _priority = State(initialValue: item.priority)
        _categoryId = State(initialValue: item.categoryId)
        _isPrivate = State(initialValue: item.isPrivate)
    }

    var body: some View {
        ScrollView {
            VStack(spacing: OffriiTheme.spacingBase) {
                // Image picker
                OffriiImagePicker(
                    selectedImage: $selectedImage,
                    existingImageUrl: item.displayImageUrl
                )
                .padding(.horizontal, OffriiTheme.spacingLG)

                OffriiCard {
                    VStack(spacing: OffriiTheme.spacingBase) {
                        OffriiTextField(
                            label: NSLocalizedString("item.name", comment: ""),
                            text: $name,
                            placeholder: NSLocalizedString("item.name", comment: ""),
                            autocapitalization: .sentences
                        )

                        OffriiTextField(
                            label: NSLocalizedString("item.description", comment: ""),
                            text: $description,
                            placeholder: NSLocalizedString("item.description", comment: ""),
                            autocapitalization: .sentences
                        )

                        OffriiTextField(
                            label: NSLocalizedString("item.estimatedPrice", comment: ""),
                            text: $estimatedPrice,
                            placeholder: "0.00",
                            keyboardType: .decimalPad
                        )

                        // Priority picker
                        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                            Text(NSLocalizedString("item.priority", comment: ""))
                                .font(OffriiTypography.subheadline)
                                .foregroundColor(OffriiTheme.textSecondary)

                            Picker("", selection: $priority) {
                                Text(NSLocalizedString("priority.low", comment: "")).tag(1)
                                Text(NSLocalizedString("priority.medium", comment: "")).tag(2)
                                Text(NSLocalizedString("priority.high", comment: "")).tag(3)
                            }
                            .pickerStyle(.segmented)
                        }

                        // Category
                        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                            Text(NSLocalizedString("item.category", comment: ""))
                                .font(OffriiTypography.subheadline)
                                .foregroundColor(OffriiTheme.textSecondary)

                            Button {
                                showCategoryPicker = true
                            } label: {
                                HStack {
                                    Text(categoryLabel)
                                        .font(OffriiTypography.body)
                                        .foregroundColor(categoryId == nil ? OffriiTheme.textMuted : OffriiTheme.text)
                                    Spacer()
                                    Image(systemName: "chevron.right")
                                        .font(.system(size: 12))
                                        .foregroundColor(OffriiTheme.textMuted)
                                }
                                .padding(.horizontal, OffriiTheme.spacingBase)
                                .padding(.vertical, 14)
                                .background(OffriiTheme.surface)
                                .cornerRadius(OffriiTheme.cornerRadiusLG)
                                .overlay(
                                    RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusLG)
                                        .strokeBorder(OffriiTheme.border, lineWidth: 1)
                                )
                            }
                        }

                        // Multi-links
                        linksSection

                        // Privacy toggle
                        Toggle(isOn: $isPrivate) {
                            VStack(alignment: .leading, spacing: 2) {
                                HStack(spacing: 4) {
                                    Image(systemName: "lock.fill")
                                        .font(.system(size: 12))
                                    Text(NSLocalizedString("wishlist.private", comment: ""))
                                        .font(OffriiTypography.body)
                                }
                                .foregroundColor(OffriiTheme.text)
                                Text(NSLocalizedString("wishlist.privateHint", comment: ""))
                                    .font(OffriiTypography.caption)
                                    .foregroundColor(OffriiTheme.textMuted)
                            }
                        }
                        .tint(OffriiTheme.primary)
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingLG)

                // Shared with section
                VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                    Text(NSLocalizedString("item.sharedWith", comment: ""))
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.textMuted)

                    ForEach(item.sharedCircles) { circle in
                        HStack(spacing: OffriiTheme.spacingSM) {
                            Text(circle.initial)
                                .font(.system(size: 11, weight: .bold))
                                .foregroundColor(.white)
                                .frame(width: 24, height: 24)
                                .background(OffriiTheme.primary)
                                .clipShape(Circle())

                            Text(circle.name)
                                .font(.system(size: 13, weight: .medium))
                                .foregroundColor(OffriiTheme.text)

                            Spacer()

                            Button {
                                Task {
                                    try? await CircleService.shared.unshareItem(circleId: circle.id, itemId: item.id)
                                }
                            } label: {
                                Image(systemName: "xmark")
                                    .font(.system(size: 10, weight: .semibold))
                                    .foregroundColor(OffriiTheme.textMuted)
                            }
                        }
                        .padding(OffriiTheme.spacingSM)
                        .background(OffriiTheme.surface)
                        .cornerRadius(OffriiTheme.cornerRadiusSM)
                    }

                    // Add row
                    Button {
                        showShareToCircle = true
                    } label: {
                        HStack(spacing: OffriiTheme.spacingSM) {
                            Image(systemName: "plus")
                                .font(.system(size: 11, weight: .semibold))
                                .foregroundColor(.white)
                                .frame(width: 24, height: 24)
                                .background(OffriiTheme.primary)
                                .clipShape(Circle())

                            Text(NSLocalizedString("share.addPeople", comment: ""))
                                .font(.system(size: 13, weight: .medium))
                                .foregroundColor(OffriiTheme.primary)
                        }
                        .padding(OffriiTheme.spacingSM)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(OffriiTheme.surface)
                        .cornerRadius(OffriiTheme.cornerRadiusSM)
                    }
                    .buttonStyle(.plain)
                }
                .padding(.horizontal, OffriiTheme.spacingLG)
                .padding(.top, OffriiTheme.spacingSM)

                OffriiButton(
                    NSLocalizedString("item.save", comment: ""),
                    isLoading: isSaving,
                    isDisabled: name.trimmingCharacters(in: .whitespaces).isEmpty
                ) {
                    Task { await save() }
                }
                .padding(.horizontal, OffriiTheme.spacingLG)
            }
            .padding(.top, OffriiTheme.spacingBase)
        }
        .background(OffriiTheme.background.ignoresSafeArea())
        .navigationTitle(NSLocalizedString("wishlist.edit", comment: ""))
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button(NSLocalizedString("common.cancel", comment: "")) {
                    dismiss()
                }
                .foregroundColor(OffriiTheme.primary)
            }
        }
        .sheet(isPresented: $showCategoryPicker) {
            CategoryPickerView(
                categories: categories,
                selectedId: $categoryId
            )
        }
        .sheet(isPresented: $showShareToCircle) {
            ShareToCircleSheet(
                itemId: item.id,
                alreadySharedCircleIds: Set(item.sharedCircles.map(\.id))
            )
            .presentationDetents([.medium])
        }
        .task {
            do {
                categories = try await CategoryService.shared.listCategories()
            } catch {}
        }
        .alert(
            NSLocalizedString("common.error", comment: ""),
            isPresented: Binding(
                get: { uploadError != nil },
                set: { if !$0 { uploadError = nil } }
            )
        ) {
            Button(NSLocalizedString("common.ok", comment: ""), role: .cancel) {}
        } message: {
            if let uploadError {
                Text(uploadError)
            }
        }
    }

    // MARK: - Links Section

    private var linksSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(NSLocalizedString("item.url", comment: ""))
                .font(OffriiTypography.subheadline)
                .foregroundColor(OffriiTheme.textSecondary)

            ForEach(links.indices, id: \.self) { index in
                HStack {
                    OffriiTextField(
                        label: "",
                        text: $links[index],
                        placeholder: "https://...",
                        keyboardType: .URL,
                        autocapitalization: .never
                    )

                    if links.count > 1 {
                        Button {
                            links.remove(at: index)
                        } label: {
                            Image(systemName: "xmark.circle.fill")
                                .foregroundColor(OffriiTheme.textMuted)
                        }
                    }
                }
            }

            if links.count < 10 {
                Button {
                    links.append("")
                } label: {
                    HStack(spacing: OffriiTheme.spacingXS) {
                        Image(systemName: "plus.circle")
                        Text(NSLocalizedString("entraide.create.addLink", comment: ""))
                    }
                    .font(OffriiTypography.footnote)
                    .foregroundColor(OffriiTheme.primary)
                }
            }
        }
    }

    // MARK: - Helpers

    private var categoryLabel: String {
        if let id = categoryId, let cat = categories.first(where: { $0.id == id }) {
            return cat.name
        }
        return NSLocalizedString("item.category", comment: "")
    }

    private func save() async {
        isSaving = true

        let trimmedName = name.trimmingCharacters(in: .whitespaces)
        let trimmedDesc = description.trimmingCharacters(in: .whitespaces)
        let price = Decimal(string: estimatedPrice.replacingOccurrences(of: ",", with: "."))
        let trimmedLinks = links.map { $0.trimmingCharacters(in: .whitespaces) }.filter { !$0.isEmpty }

        // Upload image if selected
        var imageUrl: String?
        if let image = selectedImage {
            if let data = image.compressForUpload() {
                do {
                    imageUrl = try await ItemService.shared.uploadImage(data)
                } catch {
                    uploadError = error.localizedDescription
                    isSaving = false
                    return
                }
            }
        }

        do {
            let updated = try await ItemService.shared.updateItem(
                id: item.id,
                name: trimmedName,
                description: trimmedDesc.isEmpty ? nil : trimmedDesc,
                estimatedPrice: price,
                priority: priority,
                categoryId: categoryId,
                imageUrl: imageUrl,
                links: trimmedLinks.isEmpty ? nil : trimmedLinks,
                isPrivate: isPrivate
            )
            onSave(updated)
            dismiss()

            Task {
                let withOG = await ItemService.shared.refetchIfMissingOG(updated)
                if withOG.ogImageUrl != nil {
                    onSave(withOG)
                }
            }
        } catch {
            // Could show error
        }
        isSaving = false
    }
}
