# schoolAI-backend

[![wakatime](https://wakatime.com/badge/github/Micah-Shallom/schoolAI-backend.svg)](https://wakatime.com/badge/github/Micah-Shallom/schoolAI-backend)

# Academic Content Generator API

A Rust-based web API for generating academic content, including essays, multiple-choice questions (MCQs), and presentations, powered by Retrieval-Augmented Generation (RAG). The API integrates with OpenRouter for text generation and MagicSlidesAPI for presentation creation, leveraging a PostgreSQL database, USearch for efficient chunk retrieval, and text embeddings for context-aware content generation.

## Features
- **Content Generation**:
  - Generate essays, lessons, and MCQs tailored to specific grade levels, topics, and objectives.
  - Create presentations with customizable slide counts using MagicSlidesAPI.
- **Retrieval-Augmented Generation (RAG)**:
  - Extract context from uploaded documents (.docx) using text embeddings.
  - Utilize USearch for fast and scalable retrieval of relevant document chunks.
  - Cache embeddings in PostgreSQL for efficient storage and retrieval.
- **Scalable Architecture**:
  - Built with Axum for high-performance routing.
  - Uses SeaORM for database interactions.
  - Supports CORS for cross-origin requests from frontends.
- **Secure Authentication**:
  - JWT-based authentication for protected routes (planned).
- **Extensible Design**:
  - Modular services for content generation, RAG, and external API integration.

## Architecture
The API follows a layered architecture:
- **Controllers**: Handle HTTP requests and responses (e.g., `feature_controller.rs`, `presentation_controller.rs`).
- **Services**: Business logic for content generation and RAG (e.g., `content_service.rs`, `presentation_service.rs`).
- **Models**: Data structures for requests and responses (e.g., `AcademicContentRequest`, `PresentationGeneratorResponse`).
- **Utils**: Error handling and helper functions.
- **RAG Pipeline**:
  - `fastembed` for generating text embeddings.
  - USearch for efficient similarity search and retrieval of relevant document chunks.
  - `RagStore` for managing chunks and embeddings.
  - PostgreSQL for caching embeddings with SHA256 file hashes.
- **External APIs**:
  - OpenRouter for text generation (e.g., MCQs, essays).
  - MagicSlidesAPI for presentation generation.
- **Database**: PostgreSQL with SeaORM migrations.
- **Middleware**: CORS support via `tower-http`.

## Prerequisites
- **Rust**: Version 1.70 or higher.
- **Cargo**: Rust’s package manager.
- **Environment**:
  - OpenRouter API key (`OPENROUTER_API_KEY`).
  - MagicSlidesAPI credentials (`email`, `accessId`).
  - Database URL for PostgreSQL.

## Installation
1. **Clone the Repository**:
   ```bash
   git clone https://github.com/your-username/academic-content-generator.git
   cd academic-content-generator
   ```

2. **Install Dependencies**:
   ```bash
   cargo build
   ```

3. **Run Migrations**:
   - Ensure `sea-orm-cli` is installed:
     ```bash
     cargo install sea-orm-cli
     ```
   - Run migrations:
     ```bash
     sea-orm-cli migrate up
     ```

4. **Run the Application**:
   ```bash
   cargo run
   ```
   The server will start on `http://0.0.0.0:3000`.

## Configuration
Create a `.env` file in the project root with the following variables:
```
DATABASE_URL=postgresql://user:password@localhost:5432/academic_content
OPENROUTER_API_KEY=your-openrouter-api-key
SERVER_PORT=3000
JWT_SECRET=your-jwt-secret
JWT_EXPIRATION=3600
```

- `DATABASE_URL`: PostgreSQL connection string.
- `OPENROUTER_API_KEY`: API key for OpenRouter.
- `SERVER_PORT`: Port for the Axum server.
- `JWT_SECRET`: Secret for JWT authentication (optional, for future use).
- `JWT_EXPIRATION`: JWT token expiration in seconds (optional).

## Usage
The API provides endpoints for generating academic content and presentations. Requests are sent as `multipart/form-data` with optional file uploads for RAG context. Use tools like Postman or a frontend application to interact with the API.

## API Endpoints
| Endpoint                     | Method | Description                              | Request Body (multipart/form-data)                                                                 |
|------------------------------|--------|------------------------------------------|---------------------------------------------------------------------------------------------------|
| `/api/academic-content-gen`  | POST   | Generate essays or lessons               | `grade_level`, `content_type` (e.g., "essay", "lesson"), `text_length`, `topic`, `standard_objective`, `additional_criteria` (optional), `uploaded_content` (optional) |
| `/api/mcq-gen`               | POST   | Generate multiple-choice questions (MCQs) | `grade_level`, `number_of_questions`, `topic`, `standard_objective`, `additional_criteria` (optional), `uploaded_content` (optional) |
| `/api/presentation-gen`      | POST   | Generate presentations via MagicSlidesAPI | `grade_level`, `number_of_slides`, `topic`, `standard_objective`, `additional_criteria` (optional), `uploaded_content` (optional) |

---

**Contact**: For questions or issues, reach out to [your-email@example.com](mailto:micahshallom@example.com).
