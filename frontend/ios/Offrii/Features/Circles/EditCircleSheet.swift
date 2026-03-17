import SwiftUI

struct EditCircleSheet: View {
    @Environment(\.dismiss) private var dismiss
    let circleId: UUID
    let currentName: String
    let currentImageUrl: String?
    let onSaved: () -> Void

    @State private var name: String
    @State private var selectedImage: UIImage?
    @State private var isSaving = false
    @State private var error: String?

    init(
        circleId: UUID,
        currentName: String,
        currentImageUrl: String? = nil,
        onSaved: @escaping () -> Void
    ) {
        self.circleId = circleId
        self.currentName = currentName
        self.currentImageUrl = currentImageUrl
        self.onSaved = onSaved
        _name = State(initialValue: currentName)
    }

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.background.ignoresSafeArea()

                VStack(spacing: OffriiTheme.spacingLG) {
                    // Circle image
                    OffriiImagePicker(
                        selectedImage: $selectedImage,
                        existingImageUrl: currentImageUrl.flatMap { URL(string: $0) },
                        isUploading: isSaving
                    )

                    OffriiTextField(
                        label: NSLocalizedString("circles.createName", comment: ""),
                        text: $name,
                        placeholder: NSLocalizedString(
                            "circles.createNamePlaceholder",
                            comment: ""
                        ),
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

        // Upload image if selected (stored for future use)
        var uploadedImageUrl: String?
        if let image = selectedImage {
            if let data = image.compressForUpload() {
                do {
                    uploadedImageUrl = try await APIClient.shared.uploadImage(data, type: "circle")
                } catch {
                    self.error = error.localizedDescription
                    isSaving = false
                    return
                }
            }
        }

        do {
            _ = try await CircleService.shared.updateCircle(id: circleId, name: trimmed, imageUrl: uploadedImageUrl)
            onSaved()
            dismiss()
        } catch {
            self.error = error.localizedDescription
        }
        isSaving = false
    }
}
