import PhotosUI
import SwiftUI

// MARK: - Create Wish Sheet

struct CreateWishSheet: View {
    @Environment(\.dismiss) private var dismiss

    @State private var title = ""
    @State private var description = ""
    @State private var selectedCategory: WishCategory?
    @State private var isAnonymous = false
    @State private var links: [String] = []
    @State private var showPhotoPicker = false
    @State private var selectedImage: PhotosPickerItem?
    @State private var imageUrl: String?
    @State private var uploadedImage: UIImage?
    @State private var isUploading = false
    @State private var isSubmitting = false
    @State private var error: String?

    private var isTitleValid: Bool {
        !title.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty && title.count <= 255
    }

    private var isFormValid: Bool {
        isTitleValid && selectedCategory != nil
    }

    private var isDescriptionOverLimit: Bool {
        description.count > 2000
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {
                    // Photo
                    photoSection

                    // Title
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                        Text(NSLocalizedString("entraide.create.titleLabel", comment: ""))
                            .font(OffriiTypography.subheadline)
                            .foregroundColor(OffriiTheme.text)
                        TextField(
                            NSLocalizedString("entraide.create.titlePlaceholder", comment: ""),
                            text: $title
                        )
                        .font(OffriiTypography.body)
                        .textFieldStyle(.roundedBorder)
                    }

                    // Category
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                        Text(NSLocalizedString("entraide.create.categoryLabel", comment: ""))
                            .font(OffriiTypography.subheadline)
                            .foregroundColor(OffriiTheme.text)
                        categoryPicker
                    }

                    // Description
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                        Text(NSLocalizedString("entraide.create.descriptionLabel", comment: ""))
                            .font(OffriiTypography.subheadline)
                            .foregroundColor(OffriiTheme.text)
                        TextEditor(text: $description)
                            .font(OffriiTypography.body)
                            .frame(minHeight: 100)
                            .padding(4)
                            .background(OffriiTheme.surface)
                            .cornerRadius(OffriiTheme.cornerRadiusMD)
                            .overlay(
                                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                                    .stroke(
                                        isDescriptionOverLimit ? OffriiTheme.danger : OffriiTheme.border,
                                        lineWidth: 1
                                    )
                            )
                        HStack {
                            Spacer()
                            Text("\(description.count)/2000")
                                .font(.system(size: 11))
                                .foregroundColor(isDescriptionOverLimit ? OffriiTheme.danger : OffriiTheme.textMuted)
                        }
                    }

                    // Anonymous toggle
                    Toggle(isOn: $isAnonymous) {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(NSLocalizedString("entraide.create.anonymous", comment: ""))
                                .font(OffriiTypography.body)
                                .foregroundColor(OffriiTheme.text)
                            Text(NSLocalizedString("entraide.create.anonymousHint", comment: ""))
                                .font(OffriiTypography.caption)
                                .foregroundColor(OffriiTheme.textMuted)
                        }
                    }
                    .tint(OffriiTheme.primary)

                    // Links
                    linksSection

                    // Error
                    if let error {
                        Text(error)
                            .font(OffriiTypography.caption)
                            .foregroundColor(OffriiTheme.danger)
                    }

                    // Submit
                    OffriiButton(
                        NSLocalizedString("entraide.create.submit", comment: ""),
                        variant: .primary,
                        isLoading: isSubmitting,
                        isDisabled: !isFormValid || isDescriptionOverLimit
                    ) {
                        Task { await submit() }
                    }
                }
                .padding(OffriiTheme.spacingLG)
            }
            .background(OffriiTheme.background)
            .navigationTitle(NSLocalizedString("entraide.create.title", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) { dismiss() }
                }
            }
        }
    }

    // MARK: - Photo Section

    private var photoSection: some View {
        Button { showPhotoPicker = true } label: { photoLabel }
            .buttonStyle(.plain)
            .photosPicker(
                isPresented: $showPhotoPicker,
                selection: $selectedImage,
                matching: .images
            )
            .task(id: selectedImage) {
                guard let selectedImage else { return }
                await uploadImage(selectedImage)
            }
    }

    @ViewBuilder
    private var photoLabel: some View {
        if isUploading {
            ProgressView()
                .frame(maxWidth: .infinity)
                .frame(height: 120)
                .background(OffriiTheme.surface)
                .cornerRadius(OffriiTheme.cornerRadiusLG)
        } else if let uploadedImage {
            ZStack(alignment: .topTrailing) {
                Image(uiImage: uploadedImage)
                    .resizable()
                    .aspectRatio(contentMode: .fill)
                    .frame(maxWidth: .infinity)
                    .frame(height: 120)
                    .clipShape(RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusLG))

                Button {
                    self.imageUrl = nil
                    self.uploadedImage = nil
                    self.selectedImage = nil
                } label: {
                    Image(systemName: "xmark.circle.fill")
                        .font(.system(size: 22))
                        .foregroundColor(.white)
                        .shadow(radius: 2)
                }
                .padding(OffriiTheme.spacingSM)
            }
        } else {
            VStack(spacing: OffriiTheme.spacingSM) {
                Image(systemName: "camera.fill")
                    .font(.system(size: 24))
                    .foregroundColor(OffriiTheme.textMuted)
                Text(NSLocalizedString("entraide.create.imageUrlLabel", comment: ""))
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
            }
            .frame(maxWidth: .infinity)
            .frame(height: 120)
            .background(OffriiTheme.surface)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .overlay(
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusLG)
                    .strokeBorder(OffriiTheme.border, style: StrokeStyle(lineWidth: 1, dash: [6]))
            )
        }
    }

    // MARK: - Category Picker

    private var categoryPicker: some View {
        let columns = [GridItem(.adaptive(minimum: 90), spacing: OffriiTheme.spacingSM)]
        return LazyVGrid(columns: columns, spacing: OffriiTheme.spacingSM) {
            ForEach(WishCategory.allCases) { category in
                let isSelected = selectedCategory == category
                Button {
                    withAnimation(OffriiAnimation.snappy) {
                        selectedCategory = category
                    }
                    OffriiHaptics.selection()
                } label: {
                    Text(category.label)
                        .font(.system(size: 13, weight: isSelected ? .semibold : .regular))
                        .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
                        .padding(.horizontal, OffriiTheme.spacingSM)
                        .padding(.vertical, OffriiTheme.spacingXS)
                        .frame(maxWidth: .infinity)
                        .background(isSelected ? OffriiTheme.primary : OffriiTheme.surface)
                        .cornerRadius(OffriiTheme.cornerRadiusSM)
                        .overlay(
                            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusSM)
                                .strokeBorder(isSelected ? .clear : OffriiTheme.border, lineWidth: 1)
                        )
                }
                .buttonStyle(.plain)
            }
        }
    }

    // MARK: - Links Section

    private var linksSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
            Text(NSLocalizedString("entraide.create.linksLabel", comment: ""))
                .font(OffriiTypography.subheadline)
                .foregroundColor(OffriiTheme.text)

            ForEach(links.indices, id: \.self) { index in
                HStack {
                    TextField("https://...", text: $links[index])
                        .font(OffriiTypography.body)
                        .textFieldStyle(.roundedBorder)
                        .autocapitalization(.none)
                        .keyboardType(.URL)

                    Button {
                        links.remove(at: index)
                    } label: {
                        Image(systemName: "minus.circle.fill")
                            .foregroundColor(OffriiTheme.danger)
                    }
                }
            }

            if links.count < 10 {
                Button {
                    links.append("")
                } label: {
                    Label(
                        NSLocalizedString("entraide.create.addLink", comment: ""),
                        systemImage: "plus.circle"
                    )
                    .font(OffriiTypography.footnote)
                    .foregroundColor(OffriiTheme.primary)
                }
            }
        }
    }

    // MARK: - Actions

    private func uploadImage(_ item: PhotosPickerItem) async {
        isUploading = true
        guard let data = try? await item.loadTransferable(type: Data.self),
              let uiImage = UIImage(data: data),
              let compressed = uiImage.compressForUpload()
        else {
            isUploading = false
            return
        }
        do {
            let url = try await ItemService.shared.uploadImage(compressed)
            imageUrl = url
            uploadedImage = uiImage
        } catch {
            self.error = error.localizedDescription
        }
        isUploading = false
    }

    private func submit() async {
        isSubmitting = true
        error = nil

        let validLinks = links.filter { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }

        do {
            _ = try await CommunityWishService.shared.createWish(
                title: title.trimmingCharacters(in: .whitespacesAndNewlines),
                description: description.isEmpty ? nil : description,
                category: selectedCategory ?? .other,
                isAnonymous: isAnonymous,
                imageUrl: imageUrl,
                links: validLinks.isEmpty ? nil : validLinks
            )
            OffriiHaptics.success()
            dismiss()
        } catch {
            self.error = error.localizedDescription
        }
        isSubmitting = false
    }
}
