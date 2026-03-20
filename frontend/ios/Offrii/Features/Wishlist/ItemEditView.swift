// swiftlint:disable file_length
import SwiftUI

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
    @State private var imageRemoved = false
    @State private var existingImageUrl: URL?
    @State private var circleToUnshare: SharedCircleInfo?
    @State private var showPrivateWarning = false
    @State private var sharedCircles: [SharedCircleInfo]
    @State private var didCancel = false
    @State private var didSave = false

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
        _existingImageUrl = State(initialValue: item.displayImageUrl)
        _sharedCircles = State(initialValue: item.sharedCircles)
    }

    var body: some View {
        ScrollView {
            VStack(spacing: OffriiTheme.spacingBase) {
                // Image picker
                OffriiImagePicker(
                    selectedImage: $selectedImage,
                    existingImageUrl: imageRemoved ? nil : existingImageUrl,
                    isUploading: isSaving,
                    onRemoveExisting: {
                        imageRemoved = true
                    }
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
                        Toggle(isOn: Binding(
                            get: { isPrivate },
                            set: { newValue in
                                if newValue && !item.sharedCircles.isEmpty {
                                    showPrivateWarning = true
                                } else {
                                    isPrivate = newValue
                                }
                            }
                        )) {
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
                        .alert(
                            NSLocalizedString("wishlist.privateWarning.title", comment: ""),
                            isPresented: $showPrivateWarning
                        ) {
                            Button(NSLocalizedString("wishlist.private", comment: "")) {
                                isPrivate = true
                            }
                            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {}
                        } message: {
                            Text(NSLocalizedString("wishlist.privateWarning.message", comment: ""))
                        }
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingLG)

                // Shared with section
                VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                    Text(NSLocalizedString("item.sharedWith", comment: ""))
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.textMuted)

                    ForEach(sharedCircles) { circle in
                        HStack(spacing: OffriiTheme.spacingSM) {
                            CircleAvatarBadge(circle: circle)

                            Text(circle.name)
                                .font(.system(size: 14, weight: .medium))
                                .foregroundColor(OffriiTheme.text)

                            Spacer()

                            Button {
                                circleToUnshare = circle
                            } label: {
                                Image(systemName: "xmark")
                                    .font(.system(size: 11, weight: .semibold))
                                    .foregroundColor(OffriiTheme.textMuted)
                            }
                        }
                        .padding(OffriiTheme.spacingSM)
                        .background(OffriiTheme.surface)
                        .cornerRadius(OffriiTheme.cornerRadiusMD)
                    }

                    // Add row
                    Button {
                        showShareToCircle = true
                    } label: {
                        HStack(spacing: OffriiTheme.spacingSM) {
                            Image(systemName: "plus")
                                .font(.system(size: 13, weight: .semibold))
                                .foregroundColor(.white)
                                .frame(width: 28, height: 28)
                                .background(OffriiTheme.primary)
                                .clipShape(Circle())

                            Text(NSLocalizedString("share.addPeople", comment: ""))
                                .font(.system(size: 14, weight: .medium))
                                .foregroundColor(OffriiTheme.primary)
                        }
                        .padding(OffriiTheme.spacingSM)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(OffriiTheme.surface)
                        .cornerRadius(OffriiTheme.cornerRadiusMD)
                    }
                    .buttonStyle(.plain)
                }
                .padding(.horizontal, OffriiTheme.spacingLG)
                .padding(.top, OffriiTheme.spacingSM)

                // Spacer for bottom padding
                Spacer().frame(height: OffriiTheme.spacingLG)
            }
            .padding(.top, OffriiTheme.spacingBase)
        }
        .background(OffriiTheme.background.ignoresSafeArea())
        .navigationTitle(NSLocalizedString("wishlist.edit", comment: ""))
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button(NSLocalizedString("common.cancel", comment: "")) {
                    didCancel = true
                    dismiss()
                }
                .foregroundColor(OffriiTheme.primary)
            }
            ToolbarItem(placement: .confirmationAction) {
                if isSaving {
                    ProgressView()
                } else {
                    Button(NSLocalizedString("common.ok", comment: "")) {
                        Task {
                            await save()
                            didSave = true
                            dismiss()
                        }
                    }
                    .fontWeight(.semibold)
                    .foregroundColor(OffriiTheme.primary)
                    .disabled(name.trimmingCharacters(in: .whitespaces).isEmpty)
                }
            }
        }
        .onDisappear {
            // Auto-save on swipe-dismiss (not on explicit cancel)
            if !didCancel && !didSave {
                Task { await save() }
            }
        }
        .interactiveDismissDisabled(isSaving)
        .sheet(isPresented: $showCategoryPicker) {
            CategoryPickerView(
                categories: categories,
                selectedId: $categoryId
            )
        }
        .sheet(isPresented: $showShareToCircle, onDismiss: {
            Task {
                if let updated = try? await ItemService.shared.getItem(id: item.id) {
                    sharedCircles = updated.sharedCircles
                }
            }
        }) {
            ShareToCircleSheet(
                itemId: item.id,
                alreadySharedCircleIds: Set(sharedCircles.map(\.id))
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
        .alert(
            NSLocalizedString("item.unshare.title", comment: ""),
            isPresented: Binding(
                get: { circleToUnshare != nil },
                set: { if !$0 { circleToUnshare = nil } }
            )
        ) {
            Button(NSLocalizedString("friends.remove", comment: ""), role: .destructive) {
                if let circle = circleToUnshare {
                    Task {
                        try? await CircleService.shared.unshareItem(circleId: circle.id, itemId: item.id)
                        withAnimation {
                            sharedCircles.removeAll { $0.id == circle.id }
                        }
                    }
                }
                circleToUnshare = nil
            }
            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {
                circleToUnshare = nil
            }
        } message: {
            if let circle = circleToUnshare {
                Text(String(format: NSLocalizedString("item.unshare.message", comment: ""), circle.name))
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

        // Upload image if selected, or mark as removed
        var imageUrl: String??
        if imageRemoved && selectedImage == nil {
            imageUrl = .some(nil) // Explicitly set to null
        } else if let image = selectedImage {
            if let data = image.compressForUpload() {
                do {
                    let url = try await ItemService.shared.uploadImage(data)
                    imageUrl = .some(url)
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
